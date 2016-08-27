
use std::rc::Rc;
use std::fmt;

use super::{Value, SrcLoc};

pub enum RunError {
    Panic(String, Option<SrcLoc>),
    NativeException(String),
    ScriptException(Value, SrcLoc),
}

impl RunError {
    pub fn new_script(msg : &str, loc : SrcLoc) -> RunError {
        RunError::ScriptException(Value::String(Rc::new(msg.to_string())), loc)
    }

    pub fn new_native(msg : &str) -> RunError {
        RunError::NativeException(msg.to_string())
    }

    pub fn new_panic(msg : &str, loc : Option<SrcLoc>) -> RunError {
        RunError::Panic(msg.to_string(), loc)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            RunError::Panic(ref s, None) => write!(f, "{}", s),
            RunError::Panic(ref s, Some(ref sl)) => write!(f, "{}: {}", sl, s),
            RunError::NativeException(ref s) => write!(f, "{}", s),
            RunError::ScriptException(ref v, ref sl) => write!(f, "{}: {}", sl, v),
        }
    }    
}
