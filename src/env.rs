
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use super::{Value, RunError};

pub struct Env {
    parent : Option<Rc<Env>>,
    names : RefCell<HashMap<String,usize>>,
    vals : RefCell<Vec<Value>>,
}

impl Env {
    pub fn new_global() -> Env {
        Env {
            parent : None,
            vals : RefCell::new(Vec::new()),
            names : RefCell::new(HashMap::new()),
        }
    }
    
    pub fn new(parent : Rc<Env>, names : Vec<Rc<String>>) -> Env {
        let mut hash_names = HashMap::new();
        for (i, name) in names.iter().enumerate() {
            hash_names.insert((**name).clone(), i);
        }
        
        Env {
            parent : Some(parent),
            vals : RefCell::new(Vec::new()),
            names : RefCell::new(hash_names),
        }
    }
    
    pub fn set_value(&self, env_index : usize, val_index : usize, val : Value) {
        if env_index == 0 {
            let mut vals = self.vals.borrow_mut();
            if val_index >= vals.len() {
                panic!("Env::set_value() with invalid index: {} >= {}", val_index, vals.len());
            }
            vals[val_index] = val;
        } else {
            if let Some(ref parent) = self.parent {
                parent.set_value(env_index - 1, val_index, val);
            } else {
                panic!("Env::set_value() with with invalid env_index");
            }
        }
    }
    
    pub fn get_value(&self, env_index : usize, val_index : usize) -> Value {
        if env_index == 0 {
            let vals = self.vals.borrow_mut();
            if val_index >= vals.len() {
                panic!("Env::set_value() with invalid index: {} >= {}", val_index, vals.len());
            }
            vals[val_index].clone()
        } else {
            match &self.parent {
                 &Some(ref parent) => parent.get_value(env_index - 1, val_index),
                 &None => panic!("Env::set_value() with with invalid env_index"),
            }
        }
    }

    pub fn get_name_index(&self, name : &str) -> Result<usize, RunError> {
        match self.names.borrow().get(name) {
            Some(&index) => Ok(index),
            None => Err(RunError::Fatal(format!("Variable '{}' doesn't exist", name))),
        }
    }
    
    pub fn add_name(&self, name : &str) -> usize {
        if self.parent.is_some() {
            panic!("trying to add name to non-root environment");
        }
            
        if let Some(&index) = self.names.borrow().get(name) {
            return index;
        }
            
        let new_index = self.vals.borrow().len();
        self.vals.borrow_mut().push(Value::Null);
        self.names.borrow_mut().insert(name.to_string(), new_index);
        new_index
    }
    
    fn dump(&self, f : &mut fmt::Formatter, env_index : usize) -> Result<usize, fmt::Error> {
        let env_index = match self.parent {
            Some(ref parent) => {
                let parent_index = try!(parent.dump(f, env_index));
                try!(writeln!(f, "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^"));
                parent_index + 1
            }
            None => env_index,
        };
        
        for (i, val) in self.vals.borrow().iter().enumerate() {
            match self.names.borrow().iter().find(|&(_, index)| *index==i) {
                Some((name, _)) => try!(writeln!(f, "[{}@{}] {} = {}", i, env_index, name, val)),
                None => try!(writeln!(f, "[{}@{}] ? = {}", i, env_index, val)),
            }
        }
        
        Ok(env_index)
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}

