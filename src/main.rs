pub mod parser;
mod ast;
mod src_loc;

pub use self::src_loc::SrcLoc;
use self::parser::Parser;

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
