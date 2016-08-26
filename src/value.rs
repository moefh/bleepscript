use std::fmt;
use std::rc::Rc;
use super::exec::FuncDef;
use super::{Env, RunError, NativeFunc, SrcLoc};

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(Rc<String>),
    Closure(Closure),
    NativeFunc(Rc<NativeFunc>),
}

impl Value {
    pub fn call(&self, args : &[Value], env : &Rc<Env>, loc : &SrcLoc) -> Result<Value, RunError> {
        match *self {
            Value::Closure(ref c) => c.apply(&args),
            Value::NativeFunc(ref f) => (**f)(&args, env),
            ref f => Err(RunError::new_script_exception(&format!("trying to call non-function object '{}'", f), loc.clone()))
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Value::Null            => write!(f, "null"),
            &Value::Bool(b)         => write!(f, "{}", b),
            &Value::Number(n)       => write!(f, "{}", n),
            &Value::String(ref s)   => write!(f, "{}", s),
            &Value::Closure(ref c)  => write!(f, "{}", c),
            &Value::NativeFunc(_)   => write!(f, "<native_function>"),
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
    
    pub fn apply(&self, args : &[Value]) -> Result<Value, RunError> {
        let new_env = Rc::new(Env::new(self.env.clone(), args));
        self.func.block.eval(&new_env)
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<closure@{}>", self.func.loc)
    }
}
