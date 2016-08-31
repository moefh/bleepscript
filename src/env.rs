
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use super::{Value, RunError};

/// Execution environment.
///
/// Stores local and global variable values.
/// 
/// Users usually don't need to use this directly. 
pub struct Env {
    pub parent : Option<Rc<Env>>,
    vals : RefCell<Vec<Value>>,
}

impl Env {
    pub fn new_global() -> Env {
        Env {
            parent : None,
            vals : RefCell::new(Vec::new()),
        }
    }
    
    pub fn new(parent : Rc<Env>, vals : &[Value]) -> Env {
        Env {
            parent : Some(parent),
            vals : RefCell::new(vals.to_vec()),
        }
    }
    
    pub fn size(&self) -> usize {
        self.vals.borrow().len()
    }
    
    pub fn grow(&self) {
        self.vals.borrow_mut().push(Value::Null);
    }
    
    pub fn set_value(&self, val_index : usize, env_index : usize, val : Value) -> Result<(), RunError> {
        if env_index == 0 {
            let mut vals = self.vals.borrow_mut();
            match vals.get_mut(val_index) {
                Some(v) => { *v = val; Ok(()) },
                None => Err(RunError::new_panic(None, &format!("Env::set_value() with invalid val_index: {}", val_index))),
            }
        } else {
            match self.parent {
                Some(ref parent) => parent.set_value(val_index, env_index - 1, val),
                None => Err(RunError::new_panic(None, &format!("Env::set_value() with with invalid env_index: reached root with env_index = {}", env_index)))
            }
        }
    }
    
    pub fn get_value(&self, val_index : usize, env_index : usize) -> Result<Value, RunError> {
        if env_index == 0 {
            match self.vals.borrow().get(val_index) {
                Some(v) => Ok(v.clone()),
                None => Err(RunError::new_panic(None, &format!("Env::get_value() with invalid val_index: {}", val_index)))
            }
        } else {
            match self.parent {
                 Some(ref parent) => parent.get_value(val_index, env_index - 1),
                 None => Err(RunError::new_panic(None, &format!("Env::get_value() with with invalid env_index: reached root with env_index = {}", env_index))),
            }
        }
    }

    fn dump(&self, f : &mut fmt::Formatter, env_index : usize) -> Result<(), fmt::Error> {
        if let Some(ref parent) = self.parent {
            try!(parent.dump(f, env_index + 1));
            try!(writeln!(f, "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^"));
        };
        
        for (i, val) in self.vals.borrow().iter().enumerate() {
            try!(writeln!(f, "<{}@{}> {}", i, env_index, val));
        }
        
        Ok(())
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}

