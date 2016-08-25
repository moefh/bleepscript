use std::fmt;
use std::rc::Rc;
use super::ast::FuncDef;
use super::Env;

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(Rc<String>),
    Closure(Closure),
}

impl fmt::Display for Value {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Value::Null            => write!(f, "null"),
            &Value::Bool(b)         => write!(f, "{}", b),
            &Value::Number(n)       => write!(f, "{}", n),
            &Value::String(ref s)   => write!(f, "{}", s),
            &Value::Closure(ref c)  => write!(f, "{}", c),
        }
    }
}

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
}

impl fmt::Display for Closure {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<closure@{}>", self.func.loc)
    }
}
