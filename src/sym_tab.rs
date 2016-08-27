use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use super::env::Env;

pub struct SymTab {
    parent : Option<Rc<SymTab>>,
    names : RefCell<HashMap<String,usize>>,
}

impl SymTab {
    pub fn new_global() -> SymTab {
        SymTab {
            parent : None,
            names : RefCell::new(HashMap::new()),
        }
    }
    
    pub fn new(parent : Rc<SymTab>, names : &[Rc<String>]) -> SymTab {
        let mut hash_names = HashMap::new();
        for (i, name) in names.iter().enumerate() {
            hash_names.insert((**name).clone(), i);
        }
        
        SymTab {
            parent : Some(parent),
            names : RefCell::new(hash_names),
        }
    }

    pub fn add_name(&self, name : &str) -> usize {
        if let Some(index) = self.names.borrow().get(name) {
            return *index;
        }
        
        let new_index = self.names.borrow().len();
        self.names.borrow_mut().insert(name.to_string(), new_index);
        new_index
    }

    pub fn get_name(&self, name : &str) -> Option<(usize, usize)> {
        self.get_name_internal(name, 0)
    }

    fn get_name_internal(&self, name : &str, env_index : usize) -> Option<(usize, usize)> {
        if let Some(index) = self.names.borrow().get(name) {
            return Some((*index, env_index));
        }
        
        match self.parent {
            Some(ref parent) => parent.get_name_internal(name, env_index + 1),
            None => None,
        }
    }
    
    pub fn dump_env(&self, env : &Rc<Env>) {
        for index in 0..env.size() {
            match self.names.borrow().iter().find(|&(_, i)| *i == index) {
                Some((ref name, index)) => println!("<{}:0> {} = {}", index, name, env.get_value(*index, 0)),
                None => println!("<{}:0> ? = {}", index, env.get_value(index, 0)),
            }
        }
    }
    
}