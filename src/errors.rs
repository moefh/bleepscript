
use std::rc::Rc;
use std::fmt;

use super::Value;
use super::src_loc::SrcLoc;

pub enum RunError {
    /// Fatal execution error. Aborts the script execution.
    Panic(Option<SrcLoc>, String),
    
    /// Recoverable execution error in a script.
    ScriptException(SrcLoc, Value),
    
    /// Recoverable execution error in a native function.
    /// This is necessary because native functions don't know
    /// the script source location they where called from.
    /// 
    /// Converted to `RunError::ScriptException` by the code that called
    /// the native function.
    NativeException(Value),
    
    /// Used internally to return from functions.
    Return(Value),

    /// Used internally to break from loops.
    Break,
}

impl RunError {
    /// Create a new `RunError::ScriptException`.
    pub fn new_script(loc : SrcLoc, msg : &str) -> RunError {
        RunError::ScriptException(loc, Value::String(Rc::new(msg.to_string())))
    }

    /// Create a new `RunError::NativeException` from a string.
    pub fn new_native_str(msg : &str) -> RunError {
        RunError::NativeException(Value::String(Rc::new(msg.to_string())))
    }

    /// Create a new `RunError::NativeException` from a `Value`.
    pub fn new_native_val(val : Value) -> RunError {
        RunError::NativeException(val)
    }

    /// Create new `RunError::Panic`.
    pub fn new_panic(loc : Option<SrcLoc>, msg : &str) -> RunError {
        RunError::Panic(loc, msg.to_string())
    }
    
    /// Converts from `RunError::NativeException` to `RunError::ScriptException`
    /// at the given location, or returns itself if not a `RunError::NativeException`.
    pub fn native_to_script(self, loc : &SrcLoc) -> RunError {
        match self {
            RunError::NativeException(v) => RunError::ScriptException(loc.clone(), v),
            x => x,
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            RunError::ScriptException(ref sl, ref v) => write!(f, "{}: {}", sl, v),
            RunError::Panic(None, ref s) => write!(f, "{}", s),
            RunError::Panic(Some(ref sl), ref s) => write!(f, "{}: {}", sl, s),
            RunError::NativeException(ref v) => write!(f, "{}", v),
            RunError::Break => write!(f, "break"),
            RunError::Return(ref v) => write!(f, "return {}", v),
        }
    }    
}

impl fmt::Debug for RunError {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }    
}
