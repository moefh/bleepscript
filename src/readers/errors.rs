
use std::io;
use std::fmt;

pub enum ReadError {
    GenericError(String),
    IOError(io::Error),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::GenericError(ref msg) => write!(f, "{}", msg),
            ReadError::IOError(ref err) => write!(f, "{}", err),
        }
    }
}

impl fmt::Debug for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::GenericError(ref msg) => write!(f, "{}", msg),
            ReadError::IOError(ref err) => write!(f, "{:?}", err),
        }
    }
}
