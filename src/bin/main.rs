extern crate bleepscript;

use bleepscript::*;

fn main() {
    
    let mut bleep = Bleep::new();
    if let Err(e) = bleep.load_script("scripts/main.tst") {
        println!("{}", e);
        return;
    }
    //bleep.dump_env();
    //bleep.dump_funcs();
    
    //let start = time::precise_time_ns();

    //println!("Calling script's function 'main':");
    match bleep.exec("main", &[Value::Number(42.0)]) {
        Ok(v) => println!("-> {}", v),
        Err(e) => println!("{}", e),
    }
    
    //let end = time::precise_time_ns();
    //println!("time: {}ms", (end - start) / 1_000_000);
}
