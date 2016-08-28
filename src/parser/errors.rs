
use std::fmt;
use std::io;

use super::super::src_loc::SrcLoc;

pub enum ReadError {
    Unknown(String),
    IOError(io::Error),
}

enum ParseErrorData {
    Message(String),
    ReadError(ReadError),
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
    
    pub fn from_read(loc : SrcLoc, err : ReadError) -> ParseError {
        ParseError {
            data : ParseErrorData::ReadError(err),
            loc : loc,
        }
    }
    
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data {
            ParseErrorData::Message(ref msg) => write!(f, "{}: {}", self.loc, msg),
            ParseErrorData::ReadError(ReadError::IOError(ref err)) => write!(f, "{}: {}", self.loc, err),
            ParseErrorData::ReadError(ReadError::Unknown(ref msg)) => write!(f, "{}: {}", self.loc, msg),
        }
        
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data {
            ParseErrorData::Message(ref msg) => write!(f, "{}: {}", self.loc, msg),
            ParseErrorData::ReadError(ReadError::IOError(ref err)) => write!(f, "{}: {:?}", self.loc, err),
            ParseErrorData::ReadError(ReadError::Unknown(ref msg)) => write!(f, "{}: {}", self.loc, msg),
        }
        
    }
}
