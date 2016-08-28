
use std::path;

use super::errors::ReadError;
use super::{CharReaderOpener, CharReader};

pub struct StringOpener {
    source : Option<StringReader>
}

impl StringOpener {
    pub fn for_string(string : &str) -> StringOpener {
        StringOpener {
            source : Some(StringReader::new(string))
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

pub struct StringReader {
    source : Vec<char>,
    pos : usize,
    col_num_before_newline : u32,
    line_num : u32,
    col_num : u32,
    saved : Vec<char>,
}

impl StringReader {
    pub fn new(source : &str) -> StringReader {
        StringReader {
            source : source.chars().collect(),
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

impl CharReader for StringReader {
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