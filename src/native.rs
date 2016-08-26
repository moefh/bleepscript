
use std::rc::Rc;

use super::Env;
use super::Value;
use super::RunError;

pub type NativeFunc = Fn(&[Value], &Rc<Env>) -> Result<Value,RunError>;

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
