use std::rc::Rc;

use super::super::exec;
use super::super::bytecode;
use super::super::src_loc::SrcLoc;
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
                          lhs : &Expression,
                          val : &Expression,
                          st : &mut analysis::State) -> ParseResult<exec::Expression> {
        match *lhs {
            Expression::Ident(ref id, ref loc) => {
                match sym.get_name(&*id) {
                    Some((vi, ei)) => {
                        let val = try!(val.analyze(sym, st));
                        return Ok(exec::Expression::VarAssign(exec::VarAssign::new(self.loc(), vi, ei, Box::new(val))))
                    }
                    None => return Err(ParseError::new(loc.clone(), &format!("assignment to undeclared variable '{}'", id)))
                }
            }
            
            Expression::Element(ref e) => {
                let cont = try!(e.container.analyze(sym, st));
                let index = try!(e.index.analyze(sym, st));
                let val = try!(val.analyze(sym, st));
                return Ok(exec::Expression::ElemAssign(exec::ElemAssign::new(self.loc(), Box::new(cont), Box::new(index), Box::new(val))))
            }

            Expression::BinaryOp(ref op) => {
                if *op.op == "." {
                    if let Expression::Ident(ref index, ref loc) = *op.right {
                        let cont = try!(op.left.analyze(sym, st));
                        let index = exec::Expression::String(index.clone(), loc.clone());
                        let val = try!(val.analyze(sym, st));
                        return Ok(exec::Expression::ElemAssign(exec::ElemAssign::new(self.loc(), Box::new(cont), Box::new(index), Box::new(val))))
                    }
                }
            }
            
            _ => (),
        }
        Err(ParseError::new(self.loc().clone(), "assignment to invalid target"))
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

    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        //println!("Expression::compile(): {:?}", self);

        match *self {
            Expression::Number(ref n, _) => {
                gen.add_comment(&format!("{}", *n));
                let index = gen.add_literal(Value::Number(*n));
                gen.emit_pushlit(index);
            }

            Expression::String(ref s, _) => {
                gen.add_comment(&format!("{:?}", s));
                let index = gen.add_literal(Value::String(s.clone()));
                gen.emit_pushlit(index);
            }

            Expression::Ident(ref id, ref loc) => {
                match sym.get_name(&*id) {
                    Some((vi, ei)) => {
                        gen.add_comment(&*id);
                        gen.emit_getvar(vi as u16, ei as u16)
                    },
                    None => return Err(ParseError::new(loc.clone(), &format!("name not declared: '{}'", id)))
                };
            }

            Expression::Vec(_) => {
                gen.add_comment("TODO: vec literal");
                gen.emit_halt();
            }

            Expression::Map(_) => {
                gen.add_comment("TODO: map literal");
                gen.emit_halt();
            }
            
            Expression::Element(ref e) => try!(e.compile(sym, gen)),
            
            Expression::BinaryOp(ref op) => {
                match &**op.op {
                    "=" => try!(self.compile_assignment(&*op.left, &*op.right, sym, gen)),
                    "." => try!(self.compile_dot(&*op.left, &*op.right, sym, gen)),
                    _ => try!(op.compile(sym, gen)),
                }
            }

            Expression::PrefixOp(ref op) => try!(op.compile(sym, gen)),
            
            Expression::FuncDef(_) => {
                gen.add_comment("TODO: func def");
                gen.emit_halt();
            }
            
            Expression::FuncCall(ref f) => try!(f.compile(sym, gen)),
        }
        
        Ok(())
    }
    
    pub fn compile_assignment(&self,
                              lhs : &Expression,
                              val : &Expression,
                              sym : &Rc<SymTab>,
                              gen : &mut bytecode::Program) -> ParseResult<()> {
        match *lhs {
            Expression::Ident(ref id, ref loc) => {
                match sym.get_name(&*id) {
                    Some((vi, ei)) => {
                        try!(val.compile(sym, gen));
                        gen.add_comment(&format!("{} = ...", &*id));
                        gen.emit_setvar(vi as u16, ei as u16);
                        return Ok(());
                    }
                    None => return Err(ParseError::new(loc.clone(), &format!("assignment to undeclared variable '{}'", id)))
                }
            }
            
            Expression::Element(_) => {
                gen.add_comment("TODO: assignment to element x[y]");
                gen.emit_halt();
                return Ok(());
            }

            Expression::BinaryOp(ref op) => {
                if *op.op == "." {
                    if let Expression::Ident(ref index, _) = *op.right {
                        gen.add_comment(&format!("TODO: assignment to element x.{}", index));
                        gen.emit_halt();
                        return Ok(());
                    }
                }
            }
            
            _ => (),
        }
        Err(ParseError::new(self.loc().clone(), "assignment to invalid target"))
    }

    pub fn compile_dot(&self, _lhs : &Expression, _rhs : &Expression, _sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        gen.add_comment("TODO: '.'");
        gen.emit_halt();
        Ok(())
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

    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        try!(self.container.compile(sym, gen));
        try!(self.index.compile(sym, gen));
        gen.emit_getelem();
        Ok(())
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
    
    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        try!(self.func.compile(sym, gen));
        for arg in &self.args {
            try!(arg.compile(sym, gen));
        }
        //gen.add_comment("for function call");
        //gen.emit_newenv(self.args.len() as u16);
        gen.emit_call(self.args.len() as u16);
        Ok(())
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
    
    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        let (vi, ei) = match sym.get_name(&*self.op) {
            Some((vi, ei)) => (vi, ei),
            None => return Err(ParseError::new(self.loc.clone(), &format!("operator doesn't exist: '{}'", self.op)))
        };
        gen.add_comment(&*self.op);
        gen.emit_getvar(vi as u16, ei as u16);
        
        try!(self.left.compile(sym, gen));
        try!(self.right.compile(sym, gen));

        gen.emit_call(2);
        Ok(())
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

    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        let (vi, ei) = match sym.get_name(&*self.op) {
            Some((vi, ei)) => (vi, ei),
            None => return Err(ParseError::new(self.loc.clone(), &format!("operator doesn't exist: '{}'", self.op)))
        };
        gen.add_comment(&*self.op);
        gen.emit_getvar(vi as u16, ei as u16);
        
        try!(self.arg.compile(sym, gen));

        gen.emit_call(1);
        Ok(())
    }

}

