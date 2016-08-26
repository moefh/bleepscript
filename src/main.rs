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
}
