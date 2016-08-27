//! BleepScript interpreter
//!
//! # Loading and Executing a Script
//!
//! ```
//! let mut bleep = Bleep::new();
//! bleep.load_script("test.tst").unwrap();
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

use self::src_loc::SrcLoc;
use self::sym_tab::SymTab;
use self::parser::{ops, Parser};

/// Script loader and executor.
///
/// # Example
///
/// ```
/// let mut bleep = Bleep::new();
///
/// bleep.load_script("test.tst").unwrap();
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
/// fn my_function(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
///     Ok(Value::Number(42.0))
/// }
/// ```
/// 
/// Then, use `Value::new_native_func()` to create a value for your function,
/// and `add_var()` to add it to the script global environment:
/// 
/// ```
/// fn main() {
///     let mut bleep = Bleep::new();
///
///     // add your function to the script environment
///     bleep.add_var("my_function", Value::new_native_func(my_function));
///
///     // load a script that calls your function
///     bleep.load_script("test.tst").unwrap();
///
///     // ...
/// }
/// ```
/// 
/// It's important to add your functions *before* loading a script that will
/// use them, or `load_script()` will complain about the function not existing.
///
pub struct Bleep {
    env : Rc<Env>,
    sym_tab : Rc<SymTab>,
    funcs : Vec<Rc<exec::FuncDef>>,
}

impl Bleep {
    /// Constructs a new script loader.
    pub fn new() -> Bleep {
        let mut bleep = Bleep {
            env : Rc::new(Env::new_global()),
            sym_tab : Rc::new(SymTab::new_global()),
            funcs : Vec::new(),
        };
        
        bleep.init_env();
        bleep
    }
    
    /// Loads a script file, adding its functions to the global environment.
    pub fn load_script<P: AsRef<path::Path>>(&mut self, filename : P) -> Result<(), ParseError> {
        let mut parser = self.create_parser();
        
        let ast_funcs = try!(parser.parse(filename));
        
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
        self.env.set_value(index, 0, val);
    }
    
    /// Returns the value of the given global variable, or
    /// `None` if the variable doesn't exist.
    pub fn get_var(&self, var : &str) -> Option<Value> {
        match self.sym_tab.get_name(var) {
            Some((vi, ei)) => Some(self.env.get_value(vi, ei)),
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
    
    fn create_parser(&self) -> parser::Parser {
        let mut parser = Parser::new();
        
        parser.add_op("=",   10, ops::Assoc::Right);
        parser.add_op("||",  20, ops::Assoc::Left);
        parser.add_op("&&",  30, ops::Assoc::Left);
        parser.add_op("==",  40, ops::Assoc::Right);
        parser.add_op("!=",  40, ops::Assoc::Right);
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
        
        parser
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
