use std::rc::Rc;

use super::super::src_loc::SrcLoc;
use super::super::exec;
use super::super::sym_tab::SymTab;
use super::super::Value;
use super::super::parser::{ParseResult, ParseError};
use super::analysis;
use super::FuncDef;

pub enum Expression {
    Number(f64, SrcLoc),
    String(Rc<String>, SrcLoc),
    Ident(Rc<String>, SrcLoc),
    Vec(VecLiteral),
    Map(MapLiteral),
    Element(Element),
    BinaryOp(BinaryOp),
    PrefixOp(PrefixOp),
    FuncCall(FuncCall),
    FuncDef(Rc<FuncDef>),
}

impl Expression {
    pub fn loc(&self) -> SrcLoc {
        match *self {
            Expression::Number(_, ref loc) |
            Expression::String(_, ref loc) |
            Expression::Ident(_, ref loc) => loc.clone(),
            
            Expression::Vec(ref v) => v.loc.clone(),
            Expression::Map(ref m) => m.loc.clone(),
            Expression::Element(ref e) => e.loc.clone(),
            Expression::BinaryOp(ref op) => op.loc.clone(),
            Expression::PrefixOp(ref op) => op.loc.clone(),
            Expression::FuncCall(ref f) => f.func.loc(),
            Expression::FuncDef(ref f) => f.loc.clone(),
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

            Expression::Vec(ref v) => Ok(exec::Expression::Vec(try!(v.analyze(sym, st)))),
            Expression::Map(ref m) => Ok(exec::Expression::Map(try!(m.analyze(sym, st)))),
            Expression::Element(ref e) => Ok(exec::Expression::Element(try!(e.analyze(sym, st)))),
            
            Expression::BinaryOp(ref op) => {
                match &**op.op {
                    "=" => self.analyze_assignment(sym, &*op.left, &*op.right, st),
                    "." => self.analyze_dot(sym, &*op.left, &*op.right, st),
                    _ => Ok(exec::Expression::BinaryOp(try!(op.analyze(sym, st)))),
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

    fn analyze_dot(&self,
                  sym : &Rc<SymTab>,
                  container : &Expression,
                  index : &Expression,
                  st : &mut analysis::State) -> ParseResult<exec::Expression> {
        match *index {
            Expression::Ident(ref id, ref loc) => {
                let c = try!(container.analyze(sym, st));
                let i = exec::Expression::String(id.clone(), loc.clone());
                Ok(exec::Expression::Element(exec::Element::new(self.loc(), Box::new(c), Box::new(i))))
            }
            
            _ => return Err(ParseError::new(index.loc().clone(), "attribute must be an identifier")),
        }
    }
}

// =========================================================
// MapLiteral

pub struct MapLiteral {
    pub entries : Vec<(Rc<String>, Expression)>,
    loc : SrcLoc,
}

impl MapLiteral {
    pub fn new(loc : SrcLoc, entries : Vec<(Rc<String>, Expression)>) -> MapLiteral {
        MapLiteral {
            entries : entries,
            loc : loc,
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::MapLiteral> {
        //println!("MapLiteral::analyze(): {:?}\n", self);

        let mut entries : Vec<(Value, exec::Expression)> = vec![];
        for &(ref k, ref v) in &self.entries {
            let k = Value::String(k.clone());
            let v = try!(v.analyze(sym, st));
            entries.push((k, v));
        }
        Ok(exec::MapLiteral::new(self.loc.clone(), entries))
    }
}

// =========================================================
// VecLiteral

pub struct VecLiteral {
    pub vec : Vec<Expression>,
    loc : SrcLoc,
}

impl VecLiteral {
    pub fn new(loc : SrcLoc, vec : Vec<Expression>) -> VecLiteral {
        VecLiteral {
            vec : vec,
            loc : loc,
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::VecLiteral> {
        //println!("VecLiteral::analyze(): {:?}\n", self);

        let mut entries : Vec<exec::Expression> = vec![];
        for i in &self.vec {
            entries.push(try!(i.analyze(sym, st)));
        }
        Ok(exec::VecLiteral::new(self.loc.clone(), entries))
    }
}

// =========================================================
// Element

pub struct Element {
    pub container : Box<Expression>,
    pub index : Box<Expression>,
    loc : SrcLoc,
}

impl Element {
    pub fn new(loc : SrcLoc, container : Box<Expression>, index : Box<Expression>) -> Element {
        Element {
            container : container,
            index : index,
            loc : loc,
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::Element> {
        let c = try!(self.container.analyze(sym, st));
        let i = try!(self.index.analyze(sym, st));
        Ok(exec::Element::new(self.loc.clone(), Box::new(c), Box::new(i)))
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

