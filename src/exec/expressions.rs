use std::rc::Rc;

use super::Block;
use super::super::{value, Value, RunError};
use super::super::src_loc::SrcLoc;
use super::super::env::Env;

pub enum Expression {
    Number(f64, SrcLoc),
    String(Rc<String>, SrcLoc),
    Variable(usize, usize, SrcLoc),
    FuncDef(Rc<FuncDef>),
    Assignment(Assignment),
    BinaryOp(BinaryOp),
    PrefixOp(PrefixOp),
    FuncCall(FuncCall),
}

impl Expression {
    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        match *self {
            Expression::Number(n, _)        => Ok(Value::Number(n)),
            Expression::String(ref s, _)    => Ok(Value::String(s.clone())),
            Expression::Variable(vi, ei, _) => env.get_value(vi, ei),
            Expression::FuncDef(ref f)      => Ok(FuncDef::eval(f.clone(), env)),
            Expression::Assignment(ref a)   => a.eval(env),
            Expression::BinaryOp(ref op)    => op.eval(env),
            Expression::PrefixOp(ref op)    => op.eval(env),
            Expression::FuncCall(ref f)     => f.eval(env),
        }
    }
    
    pub fn loc(&self) -> SrcLoc {
        match *self {
            Expression::Number(_, ref loc) => loc.clone(),
            Expression::String(_, ref loc) => loc.clone(),
            Expression::Variable(_, _, ref loc) => loc.clone(),
            Expression::FuncDef(ref f) => f.loc.clone(),
            Expression::Assignment(ref a) => a.loc.clone(),
            Expression::BinaryOp(ref op) => op.loc.clone(),
            Expression::PrefixOp(ref op) => op.loc.clone(),
            Expression::FuncCall(ref f) => f.func.loc(),
        }
    }

}

// =========================================================
// FuncCall

pub struct FuncCall {
    pub func : Box<Expression>,
    pub args : Vec<Expression>,
    loc : SrcLoc,
}

impl FuncCall {
    pub fn new(loc : SrcLoc, func : Box<Expression>, args : Vec<Expression>) -> FuncCall {
        FuncCall {
            func : func,
            args : args,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        let func = try!(self.func.eval(env));
        let mut args = vec![];
        for a in &self.args {
            args.push(try!(a.eval(env)));
        }
        func.call(&args, env, &self.loc)
    }
}

// =========================================================
// Assignment

pub struct Assignment {
    pub var_index : usize,
    pub env_index : usize,
    pub val : Box<Expression>,
    loc : SrcLoc,
}

impl Assignment {
    pub fn new(loc : SrcLoc, var_index : usize, env_index : usize, val : Box<Expression>) -> Assignment {
        Assignment {
            var_index : var_index,
            env_index : env_index,
            val : val,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        let val = try!(self.val.eval(env));
        try!(env.set_value(self.var_index, self.env_index, val.clone()));
        Ok(val)
    }
}

// =========================================================
// BinaryOp

pub struct BinaryOp {
    pub val_index : usize,
    pub env_index : usize,
    pub left : Box<Expression>,
    pub right : Box<Expression>,
    loc : SrcLoc,
}

impl BinaryOp {
    pub fn new(loc : SrcLoc, val_index : usize, env_index : usize, left : Box<Expression>, right : Box<Expression>) -> BinaryOp {
        BinaryOp {
            val_index : val_index,
            env_index : env_index,
            left : left,
            right : right,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        let func = try!(env.get_value(self.val_index, self.env_index));
        let left = try!(self.left.eval(env));
        let right = try!(self.right.eval(env));
        func.call(&[left, right], env, &self.loc)
    }
}

// =========================================================
// PrefixOp

pub struct PrefixOp {
    pub val_index : usize,
    pub env_index : usize,
    pub arg : Box<Expression>,
    loc : SrcLoc,
}

impl PrefixOp {
    pub fn new(loc : SrcLoc, val_index : usize, env_index : usize, arg : Box<Expression>) -> PrefixOp {
        PrefixOp {
            val_index : val_index,
            env_index : env_index,
            arg : arg,
            loc : loc,
        }
    }

    pub fn eval(&self, env : &Rc<Env>) -> Result<Value, RunError> {
        let func = try!(env.get_value(self.val_index, self.env_index));
        let arg = try!(self.arg.eval(env));
        func.call(&[arg], env, &self.loc)
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
