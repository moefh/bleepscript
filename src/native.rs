
use std::rc::Rc;

use super::Env;
use super::Value;
use super::RunError;

pub type FuncPointer = fn(&[Value], &Rc<Env>) -> Result<Value,RunError>;

#[derive(Copy)]
pub struct NativeFunc {
    pub f : FuncPointer,
}

impl NativeFunc {
    pub fn new(f : FuncPointer) -> NativeFunc {
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

// ==============================================================
// Native functions

pub fn func_printf(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            print!("\t");
        }
        print!("{}", a);
    }
    Ok(Value::Null)
}

pub fn func_dump_env(_args : &[Value], env : &Rc<Env>) -> Result<Value,RunError> {
    println!("{:?}", env);
    Ok(Value::Null)
}

pub fn func_generic(_args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    println!("called native function");
    Ok(Value::String(Rc::new("return value from native function".to_string())))
}
