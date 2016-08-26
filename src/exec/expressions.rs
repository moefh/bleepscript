use std::rc::Rc;

use super::super::SrcLoc;
use super::FuncDef;

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

// =========================================================
// FuncCall

pub struct FuncCall {
    pub func : Box<Expression>,
    pub args : Vec<Expression>,
}

impl FuncCall {
    pub fn new(func : Box<Expression>, args : Vec<Expression>) -> FuncCall {
        FuncCall {
            func : func,
            args : args,
        }
    }
}

// =========================================================
// Assignment

pub struct Assignment {
    pub var_index : usize,
    pub env_index : usize,
    pub val : Box<Expression>,
    _loc : SrcLoc,
}

impl Assignment {
    pub fn new(loc : SrcLoc, var_index : usize, env_index : usize, val : Box<Expression>) -> Assignment {
        Assignment {
            var_index : var_index,
            env_index : env_index,
            val : val,
            _loc : loc,
        }
    }
}

// =========================================================
// BinaryOp

pub struct BinaryOp {
    pub val_index : usize,
    pub env_index : usize,
    pub left : Box<Expression>,
    pub right : Box<Expression>,
    _loc : SrcLoc,
}

impl BinaryOp {
    pub fn new(loc : SrcLoc, val_index : usize, env_index : usize, left : Box<Expression>, right : Box<Expression>) -> BinaryOp {
        BinaryOp {
            val_index : val_index,
            env_index : env_index,
            left : left,
            right : right,
            _loc : loc,
        }
    }
}

// =========================================================
// PrefixOp

pub struct PrefixOp {
    pub val_index : usize,
    pub env_index : usize,
    pub arg : Box<Expression>,
    _loc : SrcLoc,
}

impl PrefixOp {
    pub fn new(loc : SrcLoc, val_index : usize, env_index : usize, arg : Box<Expression>) -> PrefixOp {
        PrefixOp {
            val_index : val_index,
            env_index : env_index,
            arg : arg,
            _loc : loc,
        }
    }
}

