//extern crate time;
extern crate bleepscript;

use std::rc::Rc;
use std::env;
use bleepscript::*;

fn test_function(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    match args.get(0) {
        Some(v) => println!("test_function() called from script with argument '{}'", v),
        None => println!("test_function() called from script with no arguments"),
    }
    Ok(Value::Null)
}

fn main() {
    let mut args = env::args();
    
    args.next();
    let script_filename = match args.next() {
        Some(f) => f,
        None => {
            println!("USAGE: bleep SCRIPT_FILENAME [SCRIPT_ARGS ...]");
            std::process::exit(1);
        }
    };
    let script_args = args.collect::<Vec<String>>();
    
    let mut bleep = Bleep::new();
    bleep.set_var("test_function", Value::new_native_func(test_function));

    if let Err(e) = bleep.compile_file(script_filename) {
        println!("{}", e);
        return;
    }
}
