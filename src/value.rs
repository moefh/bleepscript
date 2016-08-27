use std::fmt;
use std::cmp;
use std::rc::Rc;

use super::env::Env;
use super::src_loc::SrcLoc;
use super::exec::FuncDef;
use super::RunError;

/// A value of the script language.
#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(Rc<String>),
    Closure(Closure),
    NativeFunc(NativeFunc),
}

impl Value {
    pub fn new_string(s : &str) -> Value {
        Value::String(Rc::new(s.to_string()))
    }
    
    pub fn new_native_func(f : fn(&[Value], &Rc<Env>) -> Result<Value,RunError>) -> Value {
        Value::NativeFunc(NativeFunc::new(f))
    }
    
    pub fn call(&self, args : &[Value], env : &Rc<Env>, loc : &SrcLoc) -> Result<Value, RunError> {
        match *self {
            Value::Closure(ref c) => match c.apply(args) {
                Err(RunError::Return(v)) => Ok(v),
                x => x,
            },
            
            Value::NativeFunc(f) => match f.call(args, env) {
                Err(RunError::NativeException(ref str)) => Err(RunError::new_script(loc.clone(), str)),
                Err(RunError::ScriptException(_, v)) => Err(RunError::ScriptException(loc.clone(), v)),
                x => x,
            },
            
            ref f => Err(RunError::new_script(loc.clone(), &format!("trying to call non-function object '{}'", f)))
        }
    }

    pub fn is_true(&self) -> bool {
        match *self {
            Value::Null           => false,
            Value::Bool(b)        => b,
            Value::Number(n)      => n != 0.0,   // should this be always true?
            Value::String(_)      => true,
            Value::Closure(_)     => true,
            Value::NativeFunc(_)  => true,
        }
    }
    
    pub fn as_i64(&self) -> Result<i64, RunError> {
        match *self {
            Value::Null           => Err(RunError::new_native("can't convert null to i64")),
            Value::Bool(b)        => if b { Ok(1) } else { Ok(0) },
            Value::Number(n)      => Ok(n as i64),
            Value::String(_)      => Err(RunError::new_native("can't convert string to i64")),
            Value::Closure(_)     => Err(RunError::new_native("can't convert closure to i64")),
            Value::NativeFunc(_)  => Err(RunError::new_native("can't convert native function to i64")),
        }
    }

    pub fn as_f64(&self) -> Result<f64, RunError> {
        match *self {
            Value::Null           => Err(RunError::new_native("can't convert null to f64")),
            Value::Bool(b)        => if b { Ok(1.0) } else { Ok(0.0) },
            Value::Number(n)      => Ok(n),
            Value::String(_)      => Err(RunError::new_native("can't convert string to f64")),
            Value::Closure(_)     => Err(RunError::new_native("can't convert closure to f64")),
            Value::NativeFunc(_)  => Err(RunError::new_native("can't convert native function to f64")),
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Value::Null            => write!(f, "null"),
            Value::Bool(b)         => write!(f, "{}", b),
            Value::Number(n)       => write!(f, "{}", n),
            Value::String(ref s)   => write!(f, "{}", s),
            Value::Closure(ref c)  => write!(f, "{}", c),
            Value::NativeFunc(_)   => write!(f, "<native_function>"),
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
        if args.len() != self.func.num_params {
            return Err(RunError::new_script(self.func.loc.clone(),
                                            &format!("invalid number of arguments passed (expected {}, got {})",
                                                     self.func.num_params, args.len())))
        }
        let new_env = Rc::new(Env::new(self.env.clone(), args));
        self.func.block.eval(&new_env)
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

#[derive(Copy)]
pub struct NativeFunc {
    pub f : fn(&[Value], &Rc<Env>) -> Result<Value,RunError>,
}

impl NativeFunc {
    pub fn new(f : fn(&[Value], &Rc<Env>) -> Result<Value,RunError>) -> NativeFunc {
        NativeFunc {
            f : f,
        }
    }
    
    pub fn call(&self, args : &[Value], env : &Rc<Env>) -> Result<Value,RunError> {
        (self.f)(args, env)
    }
}

impl Clone for NativeFunc {
    fn clone(&self) -> NativeFunc {
        NativeFunc {
            f : self.f,
        }
    }
}

impl cmp::PartialEq for NativeFunc {
    fn eq(&self, other: &NativeFunc) -> bool {
        self as *const NativeFunc == other as *const NativeFunc
    }
}

