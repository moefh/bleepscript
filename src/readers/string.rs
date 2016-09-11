
use std::path;

use super::errors::ReadError;
use super::InputSource;

pub struct StringInputOpener {
    source : Option<Box<Iterator<Item=Result<char,ReadError>>>>
}

impl StringInputOpener {
    pub fn for_string(string : &str) -> StringInputOpener {
        StringInputOpener {
            source : Some(Box::new(StringInput::from(string)))
        }        
    }
}

impl InputSource for StringInputOpener {
    fn open(&mut self, _path : &path::Path) -> Result<Box<Iterator<Item=Result<char,ReadError>>>, ReadError> {
        match self.source.take() {
            Some(s) => Ok(Box::new(s)),
            None => Err(ReadError::GenericError("can't use 'include' from string source".to_string())),
        }
    }
}

pub struct StringInput {
    chars : Vec<char>,
    cur : usize,
}

impl StringInput {
    pub fn new(chars : Vec<char>) -> StringInput {
        StringInput {
            chars : chars,
            cur : 0,
        }
    }
}

impl Iterator for StringInput {
    type Item = Result<char,ReadError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.get(self.cur) {
            Some(&c) => {
                self.cur += 1;
                Some(Ok(c))
            }
            
            None => None,
        }
    }
}

impl<'a> From<&'a str> for StringInput {
    fn from(s : &'a str) -> StringInput {
        StringInput::new(s.chars().collect())
    }
}

impl<'a> From<&'a String> for StringInput {
    fn from(s : &'a String) -> StringInput {
        StringInput::new(s.chars().collect())
    }
}
