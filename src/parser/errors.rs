
use std::fmt;
use std::io;

use super::super::SrcLoc;

enum ParseErrorData {
    Message(String),
    IOError(io::Error),
}

pub struct ParseError {
    data : ParseErrorData,
    loc : SrcLoc,
}

impl ParseError {
    pub fn new(loc : SrcLoc, msg : &str) -> ParseError {
        ParseError {
            data : ParseErrorData::Message(msg.to_string()),
            loc : loc,
        }
    }
    
    pub fn from_io(loc : SrcLoc, err : io::Error) -> ParseError {
        ParseError {
            data : ParseErrorData::IOError(err),
            loc : loc,
        }
    }
    
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data {
            ParseErrorData::Message(ref msg) => write!(f, "{}: {}", self.loc, msg),
            ParseErrorData::IOError(ref err) => write!(f, "{}: {:?}", self.loc, err),
        }
        
    }
}
