use std::rc::Rc;

use super::super::SrcLoc;
use super::FuncDef;

pub enum Expression {
    Number(f64, SrcLoc),
    String(Rc<String>, SrcLoc),
    Ident(Rc<String>, SrcLoc),
    FuncDef(Rc<FuncDef>),
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
// BinaryOp

pub struct BinaryOp {
    pub op : Rc<String>,
    pub left : Box<Expression>,
    pub right : Box<Expression>,
    _loc : SrcLoc,
}

impl BinaryOp {
    pub fn new(loc : SrcLoc, op : Rc<String>, left : Box<Expression>, right : Box<Expression>) -> BinaryOp {
        BinaryOp {
            op : op,
            left : left,
            right : right,
            _loc : loc,
        }
    }
}

// =========================================================
// PrefixOp

pub struct PrefixOp {
    pub op : Rc<String>,
    pub arg : Box<Expression>,
    _loc : SrcLoc,
}

impl PrefixOp {
    pub fn new(loc : SrcLoc, op : Rc<String>, arg : Box<Expression>) -> PrefixOp {
        PrefixOp {
            op : op,
            arg : arg,
            _loc : loc,
        }
    }
}

