
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Assoc {
    Left,
    Right,
    Prefix,
}

#[derive(Clone)]
pub struct Operator {
    pub name : Rc<String>,
    pub prec : i32,
    pub assoc : Assoc,
}

pub struct OpTable {
    prefix : HashMap<String, Operator>,
    binary : HashMap<String, Operator>,
}

impl OpTable {
    pub fn new() -> OpTable {
        OpTable {
            prefix: HashMap::new(),
            binary: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, name : &str, prec : i32, assoc : Assoc) {
        let op = Operator {
            name : Rc::new(name.to_string()),
            prec : prec,
            assoc : assoc.clone(),
        };
        match assoc {
            Assoc::Left | Assoc::Right => self.binary.insert(name.to_string(), op),
            Assoc::Prefix => self.prefix.insert(name.to_string(), op),
        };
    }
    
    pub fn is_binary(&self, name : &str) -> bool {
        self.binary.contains_key(name)
    }
    
    pub fn is_prefix(&self, name : &str) -> bool {
        self.prefix.contains_key(name)
    }
    
    pub fn is_operator(&self, name : &str) -> bool {
        self.is_binary(name) || self.is_prefix(name)
    }
    
    pub fn get_binary(&self, name : &str) -> Option<Operator> {
        match self.binary.get(name) {
            Some(op) => Some(op.clone()),
            None => None,
        }
    }

    pub fn get_prefix(&self, name : &str) -> Option<Operator> {
        match self.prefix.get(name) {
            Some(op) => Some(op.clone()),
            None => None,
        }
    }
}
