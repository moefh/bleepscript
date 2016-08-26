
use std::rc::Rc;

use super::Expression;
use super::super::{value, Value, Env, SrcLoc, RunError};

pub enum Statement {
    Block(Block),
    Expression(Expression),
}

impl Statement {
    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        match *self {
            Statement::Block(ref b) => b.eval(env),
            Statement::Expression(ref e) => e.eval(env),
        }
    }
}

// =========================================================
// Block
pub struct Block {
    pub var : Option<Box<Expression>>,
    pub stmts : Vec<Statement>,
    _loc : SrcLoc,
}

impl Block {
    pub fn new(loc : SrcLoc, var : Option<Box<Expression>>, stmts : Vec<Statement>) -> Block {
        Block {
            var : var,
            stmts : stmts,
            _loc : loc,
        }
    }

    fn eval_stmts(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        for stmt in &self.stmts {
            try!(stmt.eval(env));
        }
        Ok(Value::Null)
    }
    
    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        if self.var.is_some() {
            let val = match self.var {
                Some(ref e) => try!(e.eval(env)),
                None => Value::Null,
            };
            self.eval_stmts(&Rc::new(Env::new(env.clone(), &[val])))
        } else {
            self.eval_stmts(env)
        }
    }
}

// =========================================================
// FuncDef
pub struct FuncDef {
    pub num_params : usize,
    pub block : Box<Block>,
    pub loc : SrcLoc,
}

impl FuncDef {
    pub fn new(loc : SrcLoc, num_params : usize, block : Box<Block>) -> FuncDef {
        FuncDef {
            num_params : num_params,
            block : block,
            loc : loc,
        }
    }
    
    pub fn eval(func : Rc<FuncDef>, env : &Rc<Env>) -> Value {
        Value::Closure(value::Closure::new(func, env.clone()))
    }
}
