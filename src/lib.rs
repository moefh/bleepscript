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
pub use self::src_loc::SrcLoc;
pub use self::parser::ParseError;
pub use self::value::Value;
pub use self::native::NativeFunc;

use self::sym_tab::SymTab;
use self::parser::Parser;
use self::parser::ops;

pub struct Bleep {
    env : Rc<Env>,
    sym_tab : Rc<SymTab>,
    funcs : Vec<Rc<exec::FuncDef>>,
}

impl Bleep {
    pub fn new() -> Bleep {
        let mut bleep = Bleep {
            env : Rc::new(Env::new_global()),
            sym_tab : Rc::new(SymTab::new_global()),
            funcs : Vec::new(),
        };
        
        bleep.init_env();
        bleep
    }
    
    pub fn set_var(&mut self, var : &str, val : Value) {
        let index = self.sym_tab.add_name(var);
        if index >= self.env.size() {
            self.env.grow();
        }
        self.env.set_value(index, 0, val);
    }
    
    pub fn get_var(&self, var : &str) -> Result<Value, RunError> {
        match self.sym_tab.get_name(var) {
            Some((vi, ei)) => Ok(self.env.get_value(vi, ei)),
            None => Err(RunError::NativeException(format!("variable not found: '{}'", var))),
        }
    }

    pub fn init_env(&mut self) {
        self.set_var("null", Value::Null);
        self.set_var("true", Value::Bool(true));
        self.set_var("false", Value::Bool(false));
        
        self.set_var("printf", Value::NativeFunc(NativeFunc::new(native::func_printf)));
        self.set_var("error", Value::NativeFunc(NativeFunc::new(native::func_error)));
        self.set_var("dump_env", Value::NativeFunc(NativeFunc::new(native::func_dump_env)));

        // TODO: actual operator functions
        let op = Value::NativeFunc(NativeFunc::new(native::func_generic));
        self.set_var("==", native::make_value(native::func_cmp_eq));
        self.set_var("!=", native::make_value(native::func_cmp_ne));
        self.set_var("<",  native::make_value(native::func_cmp_lt));
        self.set_var("<=", native::make_value(native::func_cmp_le));
        self.set_var(">",  native::make_value(native::func_cmp_gt));
        self.set_var(">=", native::make_value(native::func_cmp_ge));
        self.set_var("+",  native::make_value(native::func_num_add));
        self.set_var("-",  native::make_value(native::func_num_sub));
        self.set_var("*",  native::make_value(native::func_num_mul));
        self.set_var("/",  native::make_value(native::func_num_div));
        self.set_var("^",  native::make_value(native::func_num_pow));
        self.set_var("%",  native::make_value(native::func_num_mod));
        self.set_var("!", op.clone());
    }
    
    pub fn dump_env(&self) {
        println!("--- global environment ------------------------");
        self.sym_tab.dump_env(&self.env);
        println!("-----------------------------------------------");
    }

    pub fn dump_funcs(&self) {
        println!("--- functions ---------------------------------");
        for func in &self.funcs {
            println!("{:?}", func);
            println!("");
        }
        println!("-----------------------------------------------");
    }
    
    pub fn load_script<P: AsRef<path::Path>>(&mut self, filename : P) -> Result<(), ParseError> {
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
        
        let ast_funcs = try!(parser.parse(filename));
        
        // add all function names to the environment, so the order doesn't matter
        for ast_func in &ast_funcs {
            self.set_var(&*ast_func.name, Value::Null);
            //println!("{:?}", ast_func);
        }
        
        // analyze all functions and add the the executable versions to the environment
        for ast_func in ast_funcs {
            let func = Rc::new(try!(ast_func.analyze(&self.sym_tab)));
            let closure = exec::FuncDef::eval(func.clone(), &self.env);
            self.set_var(&*ast_func.name, closure);
            self.funcs.push(func);
        }
        Ok(())
    }
    
    pub fn exec(&self, func_name : &str, args : &[Value]) -> Result<Value, RunError> {
        let func = try!(self.get_var(func_name));
        func.call(args, &self.env, &SrcLoc::new("(no file)", 0, 0))
    }
    
}
