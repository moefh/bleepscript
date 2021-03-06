
use std::fmt;
use std::cmp;
use std::rc::Rc;

use super::FuncDef;
use super::super::Env;
use super::super::Value;
use super::super::RunError;
use super::super::src_loc::SrcLoc;

#[derive(Clone)]
pub struct Closure {
    func : Rc<FuncDef>,
    env : Rc<Env>,
}

impl Closure {
    pub fn new(func : Rc<FuncDef>, env : Rc<Env>) -> Closure {
        Closure {
            func : func,
            env : env,
        }
    }
    
    pub fn apply(&self, args : &[Value], loc : &SrcLoc) -> Result<Value, RunError> {
        if args.len() != self.func.num_params {
            return Err(RunError::new_script(loc.clone(),
                                            &format!("invalid number of arguments passed (expected {}, got {})",
                                                     self.func.num_params, args.len())))
        }
        let new_env = Rc::new(Env::new(self.env.clone(), args));
        match self.func.block.eval(&new_env) {
            Err(RunError::Return(v)) => Ok(v),
            x => x,
        }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<closure@{}>", self.func.loc)
    }
}

impl cmp::PartialEq for Closure {
    fn eq(&self, other: &Closure) -> bool {
        self as *const Closure == other as *const Closure
    }
}

