
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
    prefix : HashMap<Rc<String>, Operator>,
    binary : HashMap<Rc<String>, Operator>,
}

impl OpTable {
    pub fn new() -> OpTable {
        OpTable {
            prefix: HashMap::new(),
            binary: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, name : &str, prec : i32, assoc : Assoc) {
        let name = Rc::new(name.to_string());
        let op = Operator {
            name : name.clone(),
            prec : prec,
            assoc : assoc.clone(),
        };
        match assoc {
            Assoc::Left | Assoc::Right => self.binary.insert(name.clone(), op),
            Assoc::Prefix => self.prefix.insert(name.clone(), op),
        };
    }
    
    pub fn _is_binary(&self, name : &Rc<String>) -> bool {
        self.binary.contains_key(name)
    }
    
    pub fn _is_prefix(&self, name : &Rc<String>) -> bool {
        self.prefix.contains_key(name)
    }
    
    pub fn _is_operator(&self, name : &Rc<String>) -> bool {
        self._is_binary(name) || self._is_prefix(name)
    }
    
    pub fn get_binary(&self, name : &Rc<String>) -> Option<&Operator> {
        self.binary.get(name)
    }

    pub fn get_prefix(&self, name : &Rc<String>) -> Option<&Operator> {
        self.prefix.get(name)
    }
}
