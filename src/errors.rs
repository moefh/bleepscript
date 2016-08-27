
use std::rc::Rc;
use std::fmt;

use super::Value;
use super::src_loc::SrcLoc;

pub enum RunError {
    /// Fatal execution error. Aborts the script execution.
    Panic(Option<SrcLoc>, String),
    
    /// Recoverable execution error.
    ScriptException(SrcLoc, Value),
    
    /// Used by native functions (which don't know the source location of the calling code).
    /// Converted to ScriptException by the calling code.
    NativeException(String),
    
    /// Used internally to return from functions.
    Return(Value),

    /// Used internally to break from loops.
    Break,
}

impl RunError {
    /// Create new RunError::ScriptException.
    pub fn new_script(loc : SrcLoc, msg : &str) -> RunError {
        RunError::ScriptException(loc, Value::String(Rc::new(msg.to_string())))
    }

    /// Create new RunError::NativeException.
    pub fn new_native(msg : &str) -> RunError {
        RunError::NativeException(msg.to_string())
    }

    /// Create new RunError::Panic.
    pub fn new_panic(loc : Option<SrcLoc>, msg : &str) -> RunError {
        RunError::Panic(loc, msg.to_string())
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            RunError::Panic(None, ref s) => write!(f, "{}", s),
            RunError::Panic(Some(ref sl), ref s) => write!(f, "{}: {}", sl, s),
            RunError::NativeException(ref s) => write!(f, "{}", s),
            RunError::ScriptException(ref sl, ref v) => write!(f, "{}: {}", sl, v),
            RunError::Break => write!(f, "break"),
            RunError::Return(ref v) => write!(f, "return {}", v),
        }
    }    
}

impl fmt::Debug for RunError {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            RunError::Panic(None, ref s) => write!(f, "{}", s),
            RunError::Panic(Some(ref sl), ref s) => write!(f, "{}: {}", sl, s),
            RunError::NativeException(ref s) => write!(f, "{}", s),
            RunError::ScriptException(ref sl, ref v) => write!(f, "{}: {}", sl, v),
            RunError::Break => write!(f, "break"),
            RunError::Return(ref v) => write!(f, "return {}", v),
        }
    }    
}
