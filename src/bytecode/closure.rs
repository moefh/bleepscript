
use std::fmt;
use std::cmp;
use std::rc::Rc;

use super::super::Env;

#[derive(Clone)]
pub struct Closure {
    pub addr : u32,
    pub num_params : usize,
    pub env : Rc<Env>,
}

impl Closure {
    pub fn new(addr : u32, num_params : usize, env : Rc<Env>) -> Closure {
        Closure {
            addr : addr,
            num_params : num_params,
            env : env,
        }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<bc_closure@{}>", self.addr)
    }
}

impl cmp::PartialEq for Closure {
    fn eq(&self, other: &Closure) -> bool {
        self as *const Closure == other as *const Closure
    }
}

