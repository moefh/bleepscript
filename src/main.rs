extern crate bleepscript;

use bleepscript::*;

fn main() {

    let mut parser = Parser::new();
    parser.load_basic_ops();
    match parser.parse("scripts/main.tst") {
        Ok(funcs) => {
            for func in funcs {
                println!("{:?}", func);
            }
        }
        
        Err(e) => println!("{}", e),
    }

}
