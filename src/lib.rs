mod parser;
mod ast;
mod exec;
mod src_loc;
mod env;
mod sym_tab;
mod errors;
mod value;

use std::rc::Rc;
use std::path;

pub use self::env::Env;
pub use self::errors::RunError;
pub use self::src_loc::SrcLoc;
pub use self::parser::ParseError;
pub use self::value::Value;

use self::sym_tab::SymTab;
use self::parser::Parser;

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
        self.set_var("printf", Value::Null);
        self.set_var("+", Value::Null);
        self.set_var("-", Value::Null);
        self.set_var("*", Value::Null);
        self.set_var("/", Value::Null);
        self.set_var("^", Value::Null);
        self.set_var("!", Value::Null);
    }
    
    pub fn dump_env(&self) {
        println!("--- global environment ------------------------");
        self.sym_tab.dump_env(&self.env);
        println!("-----------------------------------------------");
    }

    pub fn dump_funcs(&self) {
        for func in &self.funcs {
            println!("{:?}", func);
            println!("");
        }
    }
    
    pub fn load_script<P: AsRef<path::Path>>(&mut self, filename : P) -> Result<(), ParseError> {
        let mut parser = Parser::new();
        parser.load_basic_ops();
        
        let ast_funcs = try!(parser.parse(filename));
        
        // add all function names to the environment, so the order doesn't matter
        for ast_func in &ast_funcs {
            self.set_var(&*ast_func.name, Value::Null);
        }
        
        // analyze all functions and add the the executable versions to the environment
        for ast_func in ast_funcs {
            let func = Rc::new(try!(ast_func.analyze(&self.sym_tab)));
            let val = Value::Closure(value::Closure::new(func.clone(), self.env.clone()));
            self.set_var(&*ast_func.name, val);
            self.funcs.push(func);
        }
        Ok(())
    }
    
}
