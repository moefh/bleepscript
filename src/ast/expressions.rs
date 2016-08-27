use std::rc::Rc;

use super::super::src_loc::SrcLoc;
use super::super::exec;
use super::super::sym_tab::SymTab;
use super::super::parser::{ParseResult, ParseError};
use super::analysis;
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

impl Expression {
    pub fn loc(&self) -> SrcLoc {
        match *self {
            Expression::Number(_, ref loc) |
            Expression::String(_, ref loc) |
            Expression::Ident(_, ref loc) => loc.clone(),
            
            Expression::FuncDef(ref f) => f.loc.clone(),
            Expression::BinaryOp(ref op) => op.loc.clone(),
            Expression::PrefixOp(ref op) => op.loc.clone(),
            Expression::FuncCall(ref f) => f.func.loc(),
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::Expression> {
        //println!("Expression::analyze(): {:?}\n", self);

        match *self {
            Expression::Number(ref n, ref loc) => Ok(exec::Expression::Number(*n, loc.clone())),
            Expression::String(ref s, ref loc) => Ok(exec::Expression::String(s.clone(), loc.clone())),

            Expression::Ident(ref id, ref loc) => {
                match sym.get_name(&*id) {
                    Some((vi, ei)) => Ok(exec::Expression::Variable(vi, ei, loc.clone())),
                    None => Err(ParseError::new(loc.clone(), &format!("name not declared: '{}'", id)))
                }
            }
            
            Expression::BinaryOp(ref op) => {
                if *op.op == "=" {
                    self.analyze_assignment(sym, &*op.left, &*op.right, st)
                } else {
                    Ok(exec::Expression::BinaryOp(try!(op.analyze(sym, st))))
                }
            }
            Expression::PrefixOp(ref op) => Ok(exec::Expression::PrefixOp(try!(op.analyze(sym, st)))),
            Expression::FuncDef(ref f) => Ok(exec::Expression::FuncDef(Rc::new(try!(f.analyze(sym, st))))),
            Expression::FuncCall(ref f) => Ok(exec::Expression::FuncCall(try!(f.analyze(sym, st)))),
        }
    }
    
    fn analyze_assignment(&self,
                          sym : &Rc<SymTab>,
                          var : &Expression,
                          val : &Expression,
                          st : &mut analysis::State) -> ParseResult<exec::Expression> {
        let (vi, ei) = match *var {
            Expression::Ident(ref id, ref loc) => {
                match sym.get_name(&*id) {
                    Some((vi, ei)) => (vi, ei),
                    None => return Err(ParseError::new(loc.clone(), &format!("assignment to undeclared variable '{}'", id)))
                }
            }
            
            _ => return Err(ParseError::new(self.loc().clone(), "assignment to invalid target"))
        };
        
        let val = try!(val.analyze(sym, st));
        Ok(exec::Expression::Assignment(exec::Assignment::new(self.loc(), vi, ei, Box::new(val))))
    }
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
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::FuncCall> {
        //println!("FuncCall::analyze(): {:?}\n", self);

        let func = try!(self.func.analyze(sym, st));
        let mut args = vec![];
        for arg in &self.args {
            args.push(try!(arg.analyze(sym, st)));
        }

        Ok(exec::FuncCall::new(func.loc(), Box::new(func), args))
    }
}

// =========================================================
// BinaryOp

pub struct BinaryOp {
    pub op : Rc<String>,
    pub left : Box<Expression>,
    pub right : Box<Expression>,
    loc : SrcLoc,
}

impl BinaryOp {
    pub fn new(loc : SrcLoc, op : Rc<String>, left : Box<Expression>, right : Box<Expression>) -> BinaryOp {
        BinaryOp {
            op : op,
            left : left,
            right : right,
            loc : loc,
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::BinaryOp> {
        //println!("BinaryOp::analyze(): {:?}\n", self);

        let (vi, ei) = match sym.get_name(&*self.op) {
            Some((vi, ei)) => (vi, ei),
            None => return Err(ParseError::new(self.loc.clone(), &format!("operator doesn't exist: '{}'", self.op)))
        };
        let left = try!(self.left.analyze(sym, st));
        let right = try!(self.right.analyze(sym, st));
        Ok(exec::BinaryOp::new(self.loc.clone(), vi, ei, Box::new(left), Box::new(right)))
    }
}

// =========================================================
// PrefixOp

pub struct PrefixOp {
    pub op : Rc<String>,
    pub arg : Box<Expression>,
    loc : SrcLoc,
}

impl PrefixOp {
    pub fn new(loc : SrcLoc, op : Rc<String>, arg : Box<Expression>) -> PrefixOp {
        PrefixOp {
            op : op,
            arg : arg,
            loc : loc,
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::PrefixOp> {
        //println!("PrefixOp::analyze(): {:?}\n", self);

        let (vi, ei) = match sym.get_name(&*self.op) {
            Some((vi, ei)) => (vi, ei),
            None => return Err(ParseError::new(self.loc.clone(), &format!("operator doesn't exist: '{}'", self.op)))
        };
        let arg = try!(self.arg.analyze(sym, st));
        Ok(exec::PrefixOp::new(self.loc.clone(), vi, ei, Box::new(arg)))
    }

}

