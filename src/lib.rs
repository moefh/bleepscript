mod parser;
mod ast;
mod src_loc;
mod env;
mod errors;
mod value;

use std::rc::Rc;
use std::path;

pub use self::env::Env;
pub use self::errors::RunError;
pub use self::src_loc::SrcLoc;
pub use self::parser::ParseError;
pub use self::value::Value;

use self::parser::Parser;

pub struct Bleep {
    env : Rc<Env>,
    funcs : Vec<ast::NamedFuncDef>,
}

impl Bleep {
    pub fn new() -> Bleep {
        Bleep {
            env : Rc::new(Env::new_global()),
            funcs : Vec::new(),
        }
    }
    
    pub fn set_var(&mut self, var : &str, val : Value) {
        let i = self.env.add_name(var);
        self.env.set_value(0, i, val);
    }
    
    pub fn get_var(&self, var : &str) {
        let i = self.env.add_name(var);
        self.env.get_value(0, i);
    }
    
    pub fn load_script<P: AsRef<path::Path>>(&mut self, filename : P) -> Result<(), ParseError> {
        let mut parser = Parser::new();
        parser.load_basic_ops();
        let funcs = try!(parser.parse(filename));
        for func in funcs {
            let val = Value::Closure(value::Closure::new(func.def.clone(), self.env.clone()));
            self.set_var(&*func.name, val);
            self.funcs.push(func);
        }
        Ok(())
    }
    
    pub fn dump_env(&self) {
        print!("{:?}", self.env);
    }

    pub fn dump_funcs(&self) {
        for func in &self.funcs {
            println!("{:?}", func);
        }
    }
    
}
