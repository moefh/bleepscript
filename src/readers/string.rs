
use std::path;

use super::errors::ReadError;
use super::{CharReaderOpener, CharReader};

pub struct StringOpener {
    source : Option<StringCharReader>
}

impl StringOpener {
    pub fn for_string(string : &str) -> StringOpener {
        StringOpener {
            source : Some(StringCharReader::from(string))
        }        
    }
}

impl CharReaderOpener for StringOpener {
    fn open(&mut self, _path : &path::Path) -> Result<Box<CharReader>, ReadError> {
        match self.source.take() {
            Some(s) => Ok(Box::new(s)),
            None => Err(ReadError::GenericError("can't use 'include' from string source".to_string())),
        }
    }
}

/// `CharReader` for `String`s.
///
/// When implementing `CharReaderOpener`, if the scripts you're reading are
/// small enough be stored as `Vec<char>`, then you can simply use a
/// `StringCharReader`.
///
/// # Examples
///
/// ```
/// use std::path;
/// use std::collections::HashMap;
/// use bleepscript::{Bleep, CharReaderOpener, StringCharReader, CharReader, ReadError};
/// 
/// struct TestOpener {
///    scripts : HashMap<path::PathBuf, String>,
/// }
/// 
/// impl TestOpener {
///    pub fn new() -> TestOpener {
///       let mut scripts = HashMap::new();
/// 
///       scripts.insert(path::PathBuf::from("first.txt"),
///                      String::from(r#"
///                          include "second.txt"
///                          function first_func() { printf("Hello\n"); }
///                      "#));
/// 
///       scripts.insert(path::PathBuf::from("second.txt"),
///                      String::from(r#"
///                          function second_func() { printf("Hello again\n"); }
///                      "#));
///
///       TestOpener {
///           scripts : scripts
///       }
///    }
/// }
/// 
/// impl CharReaderOpener for TestOpener {
///    fn open(&mut self, source : &path::Path) -> Result<Box<CharReader>, ReadError> {
///       match self.scripts.get(source) {
///           Some(str) => Ok(Box::new(StringCharReader::from(str))),
///           None => Err(ReadError::GenericError("Not found".to_string())),
///       }
///    }
/// }
///
/// let mut bleep = Bleep::new();
/// bleep.load_user("first.txt", Box::new(TestOpener::new()))
///      .expect("error loading scripts");
/// assert!(bleep.get_var("second_func").is_some());
/// ```
pub struct StringCharReader {
    source : Vec<char>,
    pos : usize,
    col_num_before_newline : u32,
    line_num : u32,
    col_num : u32,
    saved : Vec<char>,
}

impl StringCharReader {
    pub fn new(source : Vec<char>) -> StringCharReader {
        StringCharReader {
            source : source,
            saved : vec![],
            pos : 0,
            col_num_before_newline : 0,
            line_num : 1,
            col_num : 1,
        }
    }
    
    fn advance_loc(&mut self, ch : char) {
        if ch == '\n' {
            self.col_num_before_newline = self.col_num;
            self.line_num += 1;
            self.col_num = 1;
        } else {
            self.col_num += 1;
        }
    }

    fn retreat_loc(&mut self, ch : char) {
        if ch == '\n' {
            self.line_num -= 1;
            self.col_num = self.col_num_before_newline;
        } else {
            self.col_num -= 1;
        }
    }
}

impl CharReader for StringCharReader {
    fn line_num(&self) -> u32 {
        self.line_num
    }
    
    fn col_num(&self) -> u32 {
        self.col_num
    }
    
    fn getc(&mut self) -> Option<Result<char, ReadError>> {
        if let Some(ch) = self.saved.pop() {
            return Some(Ok(ch));
        }
        match self.source.get(self.pos) {
            Some(&c) => {
                self.pos += 1;
                self.advance_loc(c);
                Some(Ok(c))
            }
            
            None => None
        }
    }
    
    fn ungetc(&mut self, ch : char) {
        self.retreat_loc(ch);
        self.saved.push(ch);
    }
    
}

impl<'a> From<&'a str> for StringCharReader {
    fn from(s : &'a str) -> StringCharReader {
        StringCharReader::new(s.chars().collect())
    }
}

impl<'a> From<&'a String> for StringCharReader {
    fn from(s : &'a String) -> StringCharReader {
        StringCharReader::new(s.chars().collect())
    }
}
