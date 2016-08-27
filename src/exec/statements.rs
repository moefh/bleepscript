
use std::rc::Rc;

use super::Expression;
use super::super::{Value, RunError};
use super::super::env::Env;
use super::super::src_loc::SrcLoc;


pub enum Statement {
    Empty,
    If(IfStatement),
    While(WhileStatement),
    Break(SrcLoc),
    Return(ReturnStatement),
    Block(Block),
    Expression(Expression),
}

impl Statement {
    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        match *self {
            Statement::Empty => Ok(Value::Null),
            Statement::If(ref i) => i.eval(env),
            Statement::While(ref w) => w.eval(env),
            Statement::Break(_) => Err(RunError::Break),
            Statement::Return(ref r) => r.eval(env),
            Statement::Block(ref b) => b.eval(env),
            Statement::Expression(ref e) => e.eval(env),
        }
    }
}

// =========================================================
// Block
pub struct Block {
    pub has_var : bool,
    pub var_val : Option<Box<Expression>>,
    pub stmts : Vec<Statement>,
    _loc : SrcLoc,
}

impl Block {
    pub fn new(loc : SrcLoc, has_var : bool, var_val : Option<Box<Expression>>, stmts : Vec<Statement>) -> Block {
        Block {
            has_var : has_var,
            var_val : var_val,
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
        if self.has_var {
            let val = match self.var_val {
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
// If
pub struct IfStatement {
    pub test : Box<Expression>,
    pub true_stmt : Box<Statement>,
    pub false_stmt : Option<Box<Statement>>,
    pub loc : SrcLoc,
}

impl IfStatement {
    pub fn new(loc : SrcLoc, test : Box<Expression>, true_stmt : Box<Statement>,
               false_stmt : Option<Box<Statement>>) -> IfStatement {
        IfStatement {
            test : test,
            true_stmt : true_stmt,
            false_stmt : false_stmt,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        let test_val = try!(self.test.eval(env));
        if test_val.is_true() {
            self.true_stmt.eval(env)
        } else if let Some(ref f) = self.false_stmt {
            f.eval(env)
        } else {
            Ok(Value::Null)
        }
    }
}

// =========================================================
// While
pub struct WhileStatement {
    pub test : Box<Expression>,
    pub stmt : Box<Statement>,
    pub loc : SrcLoc,
}

impl WhileStatement {
    pub fn new(loc : SrcLoc, test : Box<Expression>, stmt : Box<Statement>) -> WhileStatement {
        WhileStatement {
            test : test,
            stmt : stmt,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        while try!(self.test.eval(env)).is_true() {
            match self.stmt.eval(env) {
                Err(RunError::Break) => break,
                Err(e) => return Err(e),
                _ => {},
            }
        }
        Ok(Value::Null)
    }
}

// =========================================================
// Return
pub struct ReturnStatement {
    pub expr : Option<Box<Expression>>,
    pub loc : SrcLoc,
}

impl ReturnStatement {
    pub fn new(loc : SrcLoc, expr : Option<Box<Expression>>) -> ReturnStatement {
        ReturnStatement {
            expr : expr,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        let val = match self.expr {
            Some(ref e) => try!(e.eval(env)),
            None => Value::Null,
        };
        Err(RunError::Return(val))
    }
}
