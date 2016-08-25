
use std::rc::Rc;

use super::{Value, SrcLoc};

pub enum RunError {
    Fatal(String),
    NativeException(String),
    ScriptException(Rc<Value>, SrcLoc),
}
