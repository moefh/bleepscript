extern crate bleepscript;

use bleepscript::*;

fn main() {
    let mut bleep = Bleep::new();
    if let Err(e) = bleep.load_script("scripts/main.tst") {
        println!("{}", e);
    }
    bleep.dump_funcs();
    bleep.dump_env();
}
