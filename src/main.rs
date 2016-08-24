mod parser;
mod ast;
mod src_loc;

pub use self::src_loc::SrcLoc;
use self::parser::Parser;

fn main() {

    let mut parser = Parser::new();
    match parser.parse("main.tst") {
        Ok(funcs) => {
            for func in funcs {
                println!("function {}({:?})", func.name, func.def.params);
            }
        }
        
        Err(e) => println!("{}", e),
    }

}
