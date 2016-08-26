extern crate bleepscript;

use bleepscript::*;

fn main() {
    let mut bleep = Bleep::new();
    if let Err(e) = bleep.load_script("scripts/main.tst") {
        println!("{}", e);
        return;
    }
    bleep.dump_env();
    bleep.dump_funcs();
    
    println!("Calling script's function 'main':");
    match bleep.exec("main", &[Value::Number(42.0)]) {
        Ok(v) => println!("Script returned '{}'", v),
        Err(e) => println!("ERROR: {}", e),
    }    
}
