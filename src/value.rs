use std::slice;
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

use super::exec;
use super::bytecode;
use super::native::NativeFunc;
use super::env::Env;
use super::src_loc::SrcLoc;
use super::RunError;

/// A value of the script language.
#[derive(Clone, PartialEq)]
pub enum Value {
    /// The null value, `null`.
    Null,
    
    /// `true` or `false`
    Bool(bool),
    
    /// Floating point number
    Number(f64),
    
    /// String
    String(Rc<String>),
    
    /// Vector
    Vec(Rc<RefCell<Vec<Value>>>),
    
    /// Map
    Map(Rc<MapValue>),
    
    /// AST Closure
    ASTClosure(exec::Closure),
    
    /// Bytecode Closure
    BCClosure(bytecode::Closure),
    
    /// Rust function 
    NativeFunc(NativeFunc),
}

impl Value {
    pub fn new_string(s : &str) -> Value {
        Value::String(Rc::new(s.to_string()))
    }
    
    pub fn new_native_func(f : fn(&[Value], &Rc<Env>) -> Result<Value,RunError>) -> Value {
        Value::NativeFunc(NativeFunc::new(f))
    }
    
    pub fn new_vector(vec : &[Value]) -> Value {
        Value::Vec(Rc::new(RefCell::new(vec.to_owned())))
    }
    
    pub fn get_element(&self, index : &Value, loc : &SrcLoc) -> Result<Value, RunError> {
        match *self {
            Value::Map(ref m) => match m.get(index) {
                Some(v) => Ok(v),
                None => Err(RunError::new_script(loc.clone(), &format!("map doesn't contain key '{}'", index)))
            },

            Value::Vec(ref v) => {
                let i = match index.as_i64() {
                    Ok(i) => i as usize,
                    Err(e) => return Err(e.native_to_script(loc)),
                };
                match v.borrow().get(i) {
                    Some(v) => Ok(v.clone()),
                    None => Err(RunError::new_script(loc.clone(), &format!("vector index out of bounds: {}", index)))
                }
            }

            _ => Err(RunError::new_script(loc.clone(), &format!("trying to read element of non-container object '{}'", self)))
        }
    }

    pub fn set_element(&mut self, index : Value, val : Value, loc : &SrcLoc) -> Result<(), RunError> {
        match *self {
            Value::Map(ref mut m) => {
                m.set(index, val);
                Ok(())
            }

            Value::Vec(ref mut v) => {
                let i = match index.as_i64() {
                    Ok(i) => i as usize,
                    Err(e) => return Err(e.native_to_script(loc)),
                };
                let mut v = v.borrow_mut();
                if i < v.len() {
                    v[i] = val.clone();
                } else if i == v.len() {
                    v.push(val.clone());
                } else {
                    return Err(RunError::new_script(loc.clone(), &format!("vector index out of bounds: {}", index)))
                }
                Ok(())
            }

            _ => Err(RunError::new_script(loc.clone(), &format!("trying to assign to element of non-container object '{}'", self)))
        }
    }
    
    pub fn is_true(&self) -> bool {
        match *self {
            Value::Null           => false,
            Value::Bool(b)        => b,
            Value::Number(n)      => n != 0.0,   // should this be always true?
            Value::String(_)      => true,
            Value::Vec(_)         => true,
            Value::Map(_)         => true,
            Value::ASTClosure(_)  => true,
            Value::BCClosure(_)   => true,
            Value::NativeFunc(_)  => true,
        }
    }
    
    pub fn as_i64(&self) -> Result<i64, RunError> {
        match *self {
            Value::Null           => Err(RunError::new_native_str("can't convert null to i64")),
            Value::Bool(b)        => if b { Ok(1) } else { Ok(0) },
            Value::Number(n)      => Ok(n as i64),
            Value::String(_)      => Err(RunError::new_native_str("can't convert string to i64")),
            Value::Vec(_)         => Err(RunError::new_native_str("can't convert vector to i64")),
            Value::Map(_)         => Err(RunError::new_native_str("can't convert map to i64")),
            Value::ASTClosure(_)  => Err(RunError::new_native_str("can't convert closure to i64")),
            Value::BCClosure(_)   => Err(RunError::new_native_str("can't convert closure to i64")),
            Value::NativeFunc(_)  => Err(RunError::new_native_str("can't convert native function to i64")),
        }
    }

    pub fn as_f64(&self) -> Result<f64, RunError> {
        match *self {
            Value::Null           => Err(RunError::new_native_str("can't convert null to f64")),
            Value::Bool(b)        => if b { Ok(1.0) } else { Ok(0.0) },
            Value::Number(n)      => Ok(n),
            Value::String(_)      => Err(RunError::new_native_str("can't convert string to f64")),
            Value::Vec(_)         => Err(RunError::new_native_str("can't convert vector to f64")),
            Value::Map(_)         => Err(RunError::new_native_str("can't convert map to f64")),
            Value::ASTClosure(_)  => Err(RunError::new_native_str("can't convert closure to f64")),
            Value::BCClosure(_)   => Err(RunError::new_native_str("can't convert closure to f64")),
            Value::NativeFunc(_)  => Err(RunError::new_native_str("can't convert native function to f64")),
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Value::Null              => write!(f, "null"),
            Value::Bool(b)           => write!(f, "{}", b),
            Value::Number(n)         => write!(f, "{}", n),
            Value::String(ref s)     => write!(f, "{}", s),
            Value::Vec(ref v)        => write!(f, "{:?}", v.borrow()),
            Value::Map(ref m)        => write!(f, "{}", m),
            Value::ASTClosure(ref c) => write!(f, "{}", c),
            Value::BCClosure(ref c)  => write!(f, "{}", c),
            Value::NativeFunc(ref n) => write!(f, "{}", n),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Value::Null              => write!(f, "null"),
            Value::Bool(b)           => write!(f, "{:?}", b),
            Value::Number(n)         => write!(f, "{:.17}", n),
            Value::String(ref s)     => write!(f, "{:?}", s),
            Value::Vec(ref v)        => write!(f, "{:?}", v.borrow()),
            Value::Map(ref m)        => write!(f, "{}", m),
            Value::ASTClosure(ref c) => write!(f, "{}", c),
            Value::BCClosure(ref c)  => write!(f, "{}", c),
            Value::NativeFunc(ref n) => write!(f, "{}", n),
        }
    }
}


#[derive(PartialEq)]
pub struct MapValue {
    entries : RefCell<Vec<(Value, Value)>>,
}

impl MapValue {
    pub fn new() -> MapValue {
        MapValue {
            entries : RefCell::new(vec![]),
        }
    }

    pub fn from_entries(entries : Vec<(Value, Value)>) -> MapValue {
        MapValue {
            entries : RefCell::new(entries),
        }
    }
    
    //pub fn insert(&mut self, key : Value, val : Value) {
    pub fn set(&self, key : Value, val : Value) {
        let mut e = self.entries.borrow_mut();
        e.retain(|&(ref k, _)| *k != key);
        e.push((key, val));
    }
    
    pub fn get(&self, key : &Value) -> Option<Value> {
        if let Some(&(_, ref v)) = self.entries.borrow().iter().find(|&&(ref k, _)| *k == *key) {
            Some(v.clone())
        } else {
            None
        }
    }
    
    pub fn iter(&self) -> MapValueIntoIterator<(Value, Value)> {
        MapValueIntoIterator {
            r : self.entries.borrow(),
        }
    }
}

pub struct MapValueIntoIterator<'a, T : 'a> {
    r: Ref<'a, Vec<T>>,
}

impl<'a, 'b : 'a, T : 'a> IntoIterator for &'b MapValueIntoIterator<'a, T> {
    type IntoIter = slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.r.iter()
    }
}

impl fmt::Display for MapValue {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{{ "));
        for &(ref k, ref v) in &self.iter() {
            try!(write!(f, "\"{}\" : {}, ", k, v));
        }
        write!(f, "}}")
    }
}
