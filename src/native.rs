
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
// Helper functions

fn get_arg(args: &[Value], index : usize) -> Result<&Value, RunError> {
    match args.get(index) {
        Some(v) => Ok(v),
        None => Err(RunError::new_native(&format!("expected argument at position {}", index+1)))
    }
}

// ==============================================================
// Native functions

pub fn func_printf(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    if let Some(&Value::String(ref fmt)) = args.get(0) {
        let mut chars = fmt.chars();
        let mut next_arg = 1;
        while let Some(ch) = chars.next() {
            if ch == '%' {
                match chars.next() {
                    Some('%') => print!("%"),
                    
                    Some('x') => { print!("{:x}", try!(try!(get_arg(args, next_arg)).as_i64())); next_arg += 1; }
                    Some('d') => { print!("{}",   try!(try!(get_arg(args, next_arg)).as_i64())); next_arg += 1; }
                    Some('f') => { print!("{}",   try!(try!(get_arg(args, next_arg)).as_f64())); next_arg += 1; }
                    Some('s') => { print!("{}",        try!(get_arg(args, next_arg)));           next_arg += 1; }
                    
                    Some(c) => return Err(RunError::new_native(&format!("invalid format specifier: {:?}", c))),
                    None    => return Err(RunError::new_native("expected format specifier")),
                };
            } else {
                print!("{}", ch);
            }
        }
        Ok(Value::Null)
    } else {
        Err(RunError::new_native("expected format string"))
    }
}

pub fn func_dump_env(_args : &[Value], env : &Rc<Env>) -> Result<Value,RunError> {
    println!("{:?}", env);
    Ok(Value::Null)
}

pub fn func_generic(_args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    println!("(called unimplemented native function, returning null)");
    Ok(Value::Null)
}
