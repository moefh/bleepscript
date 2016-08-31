
use std::path;

use super::parser::{Parser, ParseError, ops};
use super::readers;
use super::ast;

pub struct BleepLoader {
    funcs : Vec<ast::NamedFuncDef>,
}

impl BleepLoader {
    pub fn new() -> BleepLoader {
        BleepLoader {
            funcs : vec![],
        }
    }
    
    /// Loads a script from the given file.
    ///
    /// Any files included with the `include` command will be read from the filesystem,
    /// relative to the directory of the original file.
    ///
    /// # Examples
    ///
    /// ```
    /// use bleepscript::Bleep;
    ///
    /// let mut bleep = Bleep::new();
    ///
    /// match bleep.load_file("myscript.bs") {
    ///     Ok(()) => println!("Successfully loaded file!"),
    ///     Err(e) => println!("Error loading file:\n{}\n", e),
    /// }
    /// ```
    pub fn load_file<P: AsRef<path::Path>>(&mut self, filename : P) -> Result<(), ParseError> {
        //let mut parser = Parser::new(Box::new(readers::FileOpener));
        //self.init_parser(&mut parser);
        //self.funcs.append(&mut try!(parser.parse(filename)));
        self.load_user(filename, Box::new(readers::FileOpener))
    }

    /// Loads a script from the given string.
    ///
    /// Scripts loaded from strings can't contain `include` commands, because
    /// the string is the only available source.
    ///
    /// # Examples
    /// ```
    /// use bleepscript::{Bleep, Value};
    ///
    /// let mut bleep = Bleep::new();
    ///
    /// bleep.load_string(r#"function test() { printf("Hello, world!\n"); return 42; }"#)
    ///      .expect("Error loading string");
    /// 
    /// let result = bleep.call_function("test", &[]).expect("Error in function test()");
    /// assert_eq!(result, Value::Number(42.0));
    /// ```
    pub fn load_string(&mut self, string : &str) -> Result<(), ParseError> {
        //let mut parser = Parser::new(Box::new(readers::StringOpener::for_string(string)));
        //self.init_parser(&mut parser);
        //self.funcs.append(&mut try!(parser.parse("(string)")));
        self.load_user("(string)", Box::new(readers::StringOpener::for_string(string)))
    }

    /// Loads a script from the given source, using the given source opener.
    ///
    /// The source opener will be used to open the given source and any other sources
    /// included by the script. 
    pub fn load_user<P: AsRef<path::Path>>(&mut self, source : P, source_opener : Box<readers::CharReaderOpener>) -> Result<(), ParseError> {
        let mut parser = Parser::new(source_opener);
        self.init_parser(&mut parser);
        self.funcs.append(&mut try!(parser.parse(source)));
        Ok(())
    }

    fn init_parser(&self, parser : &mut Parser) {
        parser.add_op("=",   10, ops::Assoc::Right);
        parser.add_op("||",  20, ops::Assoc::Left);
        parser.add_op("&&",  30, ops::Assoc::Left);
        parser.add_op("==",  40, ops::Assoc::Left);
        parser.add_op("!=",  40, ops::Assoc::Left);
        parser.add_op("<",   50, ops::Assoc::Left);
        parser.add_op(">",   50, ops::Assoc::Left);
        parser.add_op("<=",  50, ops::Assoc::Left);
        parser.add_op(">=",  50, ops::Assoc::Left);
        parser.add_op("+",   60, ops::Assoc::Left);
        parser.add_op("-",   60, ops::Assoc::Left);
        parser.add_op("*",   70, ops::Assoc::Left);
        parser.add_op("/",   70, ops::Assoc::Left);
        parser.add_op("%",   70, ops::Assoc::Left);
        parser.add_op("-",   80, ops::Assoc::Prefix);
        parser.add_op("!",   80, ops::Assoc::Prefix);
        parser.add_op("^",   90, ops::Assoc::Right);
        parser.add_op(".", 1001, ops::Assoc::Left);

        parser.set_element_index_prec(1000);
        parser.set_function_call_prec(1000);
    }
    
    pub fn get_functions(self) -> Vec<ast::NamedFuncDef> {
        self.funcs
    }

}
