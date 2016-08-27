
use std::cmp;
use std::rc::Rc;

use super::{Env,Value,RunError,SrcLoc};

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

impl cmp::PartialEq for NativeFunc {
    fn eq(&self, other: &NativeFunc) -> bool {
        self as *const NativeFunc == other as *const NativeFunc
    }
}

pub fn make_value(f : FuncPointer) -> Value {
    Value::NativeFunc(NativeFunc::new(f))
}

// ==============================================================
// Helper functions

fn get_arg(args: &[Value], index : usize) -> Result<&Value, RunError> {
    match args.get(index) {
        Some(v) => Ok(v),
        None => Err(RunError::new_native(&format!("expected argument at position {}", index+1)))
    }
}

fn cmp_eq(args : &[Value]) -> Result<bool,RunError> {
    let left = try!(get_arg(args, 0));
    let right = try!(get_arg(args, 1));
    
    let b = match (left, right) {
        (&Value::Null,              &Value::Null             ) => true,
        (&Value::Bool(l),           &Value::Bool(r)          ) => l == r,
        (&Value::Number(l),         &Value::Number(r)        ) => l == r,
        (&Value::String(ref l),     &Value::String(ref r)    ) => l == r,
        (&Value::Closure(ref l),    &Value::Closure(ref r)   ) => l == r,
        (&Value::NativeFunc(ref l), &Value::NativeFunc(ref r)) => l == r,
        _ => false,
    };
    Ok(b)
}

fn cmp_order(args : &[Value],
             num_cmp : fn(f64,f64)->bool,
             str_cmp : fn(&str,&str)->bool) -> Result<bool,RunError> {
    let left = try!(get_arg(args, 0));
    let right = try!(get_arg(args, 1));
    
    let b = match (left, right) {
        (&Value::Number(l),      &Value::Number(r)    ) => num_cmp(l, r),
        (&Value::String(ref l),  &Value::String(ref r)) => str_cmp(l, r),
        _ => false,
    };
    Ok(b)
}

fn cmp_lt_num(l:f64, r:f64) -> bool { l< r }
fn cmp_le_num(l:f64, r:f64) -> bool { l<=r }

fn cmp_lt_str(l:&str, r:&str) -> bool { l< r }
fn cmp_le_str(l:&str, r:&str) -> bool { l<=r }

fn bin_arithmetic(args : &[Value], op : fn(f64,f64)->f64, name : &str) -> Result<f64,RunError> {
    let left = try!(get_arg(args, 0));
    let right = try!(get_arg(args, 1));
    match (left, right) {
        (&Value::Number(l), &Value::Number(r)) => Ok(op(l, r)),
        _ => Err(RunError::new_native(&format!("invalid arguments for '{}'", name)))
    }
}

fn num_add(l:f64, r:f64)->f64 { l+r }
fn num_mul(l:f64, r:f64)->f64 { l*r }
fn num_sub(l:f64, r:f64)->f64 { l-r }
fn num_div(l:f64, r:f64)->f64 { l/r }
fn num_pow(l:f64, r:f64)->f64 { l.powf(r) }
fn num_mod(l:f64, r:f64)->f64 { l - (l/r).trunc() * r }

fn un_arithmetic(args : &[Value], op : fn(f64)->f64, name : &str) -> Result<f64,RunError> {
    let arg = try!(get_arg(args, 0));
    match arg {
        &Value::Number(x) => Ok(op(x)),
        _ => Err(RunError::new_native(&format!("invalid argument for '{}'", name)))
    }
}

fn num_neg(x:f64)->f64 { -x }

// ==============================================================
// Native functions

pub fn func_generic(_args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    println!("(called unimplemented native function, returning null)");
    Ok(Value::Null)
}

pub fn func_dump_env(_args : &[Value], env : &Rc<Env>) -> Result<Value,RunError> {
    println!("{:?}", env);
    Ok(Value::Null)
}

pub fn func_error(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    if let Some(v) = args.get(0) {
        Err(RunError::ScriptException(v.clone(), SrcLoc::new("",0,0)))
    } else {
        Ok(Value::Null)
    }
}

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

// ==
pub fn func_cmp_eq(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Bool(try!(cmp_eq(args))))
}

// !=
pub fn func_cmp_ne(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Bool(! try!(cmp_eq(args))))
}

// <
pub fn func_cmp_lt(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Bool(try!(cmp_order(args, cmp_lt_num, cmp_lt_str))))
}

// <=
pub fn func_cmp_le(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Bool(try!(cmp_order(args, cmp_le_num, cmp_le_str))))
}

// >   note: a > b  <=>  ! (a <= b)
pub fn func_cmp_gt(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Bool(! try!(cmp_order(args, cmp_le_num, cmp_le_str))))
}

// >=  note: a >= b  <=>  ! (a < b)
pub fn func_cmp_ge(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Bool(! try!(cmp_order(args, cmp_lt_num, cmp_lt_str))))
}

// +
pub fn func_num_add(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_add, "+"))))
}

// -
pub fn func_num_sub(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    if args.len() == 1 {
        return Ok(Value::Number(try!(un_arithmetic(args, num_neg, "-"))));
    }
    Ok(Value::Number(try!(bin_arithmetic(args, num_sub, "-"))))
}

// *
pub fn func_num_mul(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_mul, "*"))))
}

// /
pub fn func_num_div(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_div, "/"))))
}

// ^
pub fn func_num_pow(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_pow, "^"))))
}

// %
pub fn func_num_mod(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_mod, "%"))))
}

