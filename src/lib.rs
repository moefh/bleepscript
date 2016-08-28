//! BleepScript interpreter
//!
//! # Loading and Executing a Script
//!
//! ```
//! use bleepscript::Bleep;
//!
//! let mut bleep = Bleep::new();
//! bleep.load_file("scripts/test.tst").unwrap();
//! bleep.call_function("main", &[]).unwrap();
//! ```
//!
//! # Example Script
//! 
//! ```text
//! function main() {
//!     printf("Hello, world!\n");
//!     return 42;
//! }
//! ```
//! 

mod errors;
mod readers;
mod parser;
mod ast;
mod exec;
mod src_loc;
mod env;
mod sym_tab;
mod value;
mod native;

use std::rc::Rc;
use std::path;

pub use self::env::Env;
pub use self::errors::RunError;
pub use self::parser::ParseError;
pub use self::value::Value;
pub use self::readers::{CharReader, CharReaderOpener, ReadError};

use self::src_loc::SrcLoc;
use self::sym_tab::SymTab;
use self::parser::{ops, Parser};

/// Script loader and executor.
///
/// # Example
///
/// ```
/// use bleepscript::Bleep;
///
/// let mut bleep = Bleep::new();
///
/// bleep.load_file("scripts/test.tst").unwrap();
///
/// let ret = bleep.call_function("main", &[]).unwrap();
/// println!("function returned {}", ret);
/// ```
///
/// # Adding User Functions
///
/// To make a Rust function callable from a script, first create your Rust
/// function with a signature like this:
/// 
/// ```
/// use std::rc;
/// use bleepscript::{Value, Env, RunError};
///
/// fn my_function(_args : &[Value], _env : &rc::Rc<Env>) -> Result<Value, RunError> {
///     Ok(Value::Number(42.0))
/// }
/// ```
/// 
/// Then, use `Value::new_native_func()` to create a value for your function,
/// and `set_var()` to add it to the script global environment:
/// 
/// ```
/// # use std::rc;
/// # use bleepscript::{Env, RunError, Value};
/// # fn my_function(_args : &[Value], _env : &rc::Rc<Env>) -> Result<Value, RunError> {
/// #     Ok(Value::Number(42.0))
/// # }
/// #
/// # use bleepscript::Bleep;
/// # fn main() {
/// #    let mut bleep = Bleep::new();
/// bleep.set_var("my_function", Value::new_native_func(my_function));
/// # }
/// ```
/// 
/// It's important to add your functions *before* loading a script that will
/// use them, or the `load_*()` functions will complain about the function not
/// existing.
///
pub struct Bleep {
    env : Rc<Env>,
    sym_tab : Rc<SymTab>,
    funcs : Vec<Rc<exec::FuncDef>>,
}

impl Bleep {
    /// Constructs a new script interpreter.
    ///
    /// The environment will be prepared with the basic Bleep functions.
    ///
    /// # Examples
    /// ```
    /// use bleepscript::Bleep;
    ///
    /// let mut bleep = Bleep::new();
    /// ```
    pub fn new() -> Bleep {
        let mut bleep = Bleep {
            env : Rc::new(Env::new_global()),
            sym_tab : Rc::new(SymTab::new_global()),
            funcs : Vec::new(),
        };
        
        bleep.init_env();
        bleep
    }
    
    /// Loads a script from the given file.
    ///
    /// Any files included with the `include` command will be read from the filesystem,
    /// relative to the directory of the original file.
    ///
    /// # Examples
    ///
    /// ```
    /// use bleepscript::Bleep;
    ///
    /// let mut bleep = Bleep::new();
    ///
    /// match bleep.load_file("myscript.bs") {
    ///     Ok(()) => println!("Successfully loaded file!"),
    ///     Err(e) => println!("Error loading file:\n{}\n", e),
    /// }
    /// ```
    pub fn load_file<P: AsRef<path::Path>>(&mut self, filename : P) -> Result<(), ParseError> {
        let mut parser = Parser::new(Box::new(readers::FileOpener));
        self.init_parser(&mut parser);
        self.load_functions(try!(parser.parse(filename)))
    }

    /// Loads a script from the given string.
    ///
    /// Scripts loaded from strings can't contain `include` commands, because
    /// the string is the only available source.
    ///
    /// # Examples
    /// ```
    /// use bleepscript::{Bleep, Value};
    ///
    /// let mut bleep = Bleep::new();
    ///
    /// bleep.load_string(r#"function test() { printf("Hello, world!\n"); return 42; }"#)
    ///      .expect("Error loading string");
    /// 
    /// let result = bleep.call_function("test", &[]).expect("Error in function test()");
    /// assert_eq!(result, Value::Number(42.0));
    /// ```
    pub fn load_string(&mut self, string : &str) -> Result<(), ParseError> {
        let mut parser = Parser::new(Box::new(readers::StringOpener::for_string(string)));
        self.init_parser(&mut parser);
        self.load_functions(try!(parser.parse("(string)")))
    }

    /// Loads a script from the given source, using the given source opener.
    ///
    /// The source opener will be used to open the given source and any other sources
    /// included by the script. 
    pub fn load_user<P: AsRef<path::Path>>(&mut self, source : P, source_opener : Box<CharReaderOpener>) -> Result<(), ParseError> {
        let mut parser = Parser::new(source_opener);
        self.init_parser(&mut parser);
        self.load_functions(try!(parser.parse(source)))
    }
    
    fn load_functions(&mut self, ast_funcs : Vec<ast::NamedFuncDef>) -> Result<(), ParseError> {
        // add all function names to the environment, so the order of
        // function definitions doesn't matter
        for ast_func in &ast_funcs {
            self.set_var(&*ast_func.name, Value::Null);
            //println!("{:?}", ast_func);
        }
        
        // analyze each function and add the the result to the environment
        for ast_func in ast_funcs {
            let func = Rc::new(try!(ast_func.analyze(&self.sym_tab, &mut ast::analysis::State::new())));
            let closure = exec::FuncDef::eval(func.clone(), &self.env);
            self.set_var(&*ast_func.name, closure);
            //println!("{:?}", func);
            self.funcs.push(func);
        }
        Ok(())
    }
    
    /// Calls the specified script function with the given arguments.
    ///
    /// The number of arguments must match the function definition, or an error will be returned.
    /// 
    /// Returns the value returned by the function (`Value::Null` if the function didn't return
    /// a value), or any error encountered during execution.
    pub fn call_function(&self, func_name : &str, args : &[Value]) -> Result<Value, RunError> {
        let loc = SrcLoc::new("(no file)", 0, 0);
        let func = match self.get_var(func_name) {
            Some(f) => f,
            None => return Err(RunError::new_script(loc, &format!("function not found: '{}'", func_name))),
        };
        func.call(args, &self.env, &loc)
    }

    /// Sets the value of the given global variable to the given value.
    /// If the variable doesn't exist, it will be created.
    pub fn set_var(&mut self, var : &str, val : Value) {
        let index = self.sym_tab.add_name(var);
        if index >= self.env.size() {
            self.env.grow();
        }
        if let Err(e) = self.env.set_value(index, 0, val) {
            self.dump_env();
            panic!("Error setting variable: {}", e);
        }
    }
    
    /// Returns the value of the given global variable, or
    /// `None` if the variable doesn't exist.
    pub fn get_var(&self, var : &str) -> Option<Value> {
        match self.sym_tab.get_name(var) {
            Some((vi, ei)) => {
                match self.env.get_value(vi, ei) {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            }
            
            None => None,
        }
    }

    fn init_env(&mut self) {
        self.set_var("null", Value::Null);
        self.set_var("true", Value::Bool(true));
        self.set_var("false", Value::Bool(false));
        
        self.set_var("printf", Value::new_native_func(native::func_printf));
        self.set_var("error", Value::new_native_func(native::func_error));
        self.set_var("dump_env", Value::new_native_func(native::func_dump_env));

        self.set_var("!",  Value::new_native_func(native::func_logic_not));
        self.set_var("==", Value::new_native_func(native::func_cmp_eq));
        self.set_var("!=", Value::new_native_func(native::func_cmp_ne));
        self.set_var("<",  Value::new_native_func(native::func_cmp_lt));
        self.set_var("<=", Value::new_native_func(native::func_cmp_le));
        self.set_var(">",  Value::new_native_func(native::func_cmp_gt));
        self.set_var(">=", Value::new_native_func(native::func_cmp_ge));
        self.set_var("+",  Value::new_native_func(native::func_num_add));
        self.set_var("-",  Value::new_native_func(native::func_num_sub));
        self.set_var("*",  Value::new_native_func(native::func_num_mul));
        self.set_var("/",  Value::new_native_func(native::func_num_div));
        self.set_var("^",  Value::new_native_func(native::func_num_pow));
        self.set_var("%",  Value::new_native_func(native::func_num_mod));
    }
    
    fn init_parser(&self, parser : &mut parser::Parser) {
        parser.add_op("=",   10, ops::Assoc::Right);
        parser.add_op("||",  20, ops::Assoc::Left);
        parser.add_op("&&",  30, ops::Assoc::Left);
        parser.add_op("==",  40, ops::Assoc::Left);
        parser.add_op("!=",  40, ops::Assoc::Left);
        parser.add_op("<",   50, ops::Assoc::Left);
        parser.add_op(">",   50, ops::Assoc::Left);
        parser.add_op("<=",  50, ops::Assoc::Left);
        parser.add_op(">=",  50, ops::Assoc::Left);
        parser.add_op("+",   60, ops::Assoc::Left);
        parser.add_op("-",   60, ops::Assoc::Left);
        parser.add_op("*",   70, ops::Assoc::Left);
        parser.add_op("/",   70, ops::Assoc::Left);
        parser.add_op("%",   70, ops::Assoc::Left);
        parser.add_op("-",   80, ops::Assoc::Prefix);
        parser.add_op("!",   80, ops::Assoc::Prefix);
        parser.add_op("^",   90, ops::Assoc::Right);
        parser.add_op(".", 1001, ops::Assoc::Left);

        parser.set_element_index_prec(1000);
        parser.set_function_call_prec(1000);
    }
    
    /// Dumps the global environment (used for debugging).
    pub fn dump_env(&self) {
        println!("--- global environment ------------------------");
        self.sym_tab.dump_env(&self.env);
        println!("-----------------------------------------------");
    }

    /// Dumps all loaded functions (used for debugging).
    pub fn dump_funcs(&self) {
        println!("--- functions ---------------------------------");
        for func in &self.funcs {
            println!("{:?}", func);
            println!("");
        }
        println!("-----------------------------------------------");
    }

}
