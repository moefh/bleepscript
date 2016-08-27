
use std;
use std::rc::Rc;
use super::super::SrcLoc;
use super::Expression;

use super::super::exec;
use super::super::sym_tab::SymTab;
use super::super::parser::{ParseResult, ParseError};
use super::analysis;

pub enum Statement {
    Expression(Expression),
    Empty,
    Block(Block),
    VarDecl(VarDecl),
    If(IfStatement),
    While(WhileStatement),
    Break(SrcLoc),
    Return(ReturnStatement),
}

impl Statement {
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::Statement> {
        match *self {
            Statement::Empty => Ok(exec::Statement::Empty),
            Statement::VarDecl(_) => panic!("trying to analyze variable declaration"),
            Statement::Expression(ref e) => Ok(exec::Statement::Expression(try!(e.analyze(sym, st)))),
            Statement::Block(ref b) => Ok(exec::Statement::Block(try!(b.analyze(sym, st)))),
            Statement::If(ref i) => Ok(exec::Statement::If(try!(i.analyze(sym, st)))),
            Statement::While(ref w) => Ok(exec::Statement::While(try!(w.analyze(sym, st)))),
            Statement::Return(ref r) => Ok(exec::Statement::Return(try!(r.analyze(sym, st)))),
            Statement::Break(ref l) => {
                if try!(st.allow_break(l)) {
                    Ok(exec::Statement::Break(l.clone()))
                } else {
                    Err(ParseError::new(l.clone(), "'break' not allowed here"))
                }
            }
        }
    }
}

// =========================================================
// Block
pub struct Block {
    pub stmts : Vec<Statement>,
    loc : SrcLoc,
}

impl Block {
    pub fn new(loc : SrcLoc, stmts : Vec<Statement>) -> Block {
        Block {
            stmts : stmts,
            loc : loc,
        }
    }

    pub fn analyze_stmts<'a>(&self,
                             sym : &Rc<SymTab>,
                             mut iter : std::slice::Iter<'a, Statement>,
                             st : &mut analysis::State) -> ParseResult<Vec<exec::Statement>> {
        let mut ret = Vec::new();
        
        while let Some(stmt) = iter.next() {
            match stmt {
                &Statement::VarDecl(ref decl) => {
                    let val = match decl.val {
                        Some(ref e) => Some(Box::new(try!(e.analyze(sym, st)))),
                        None => None,
                    };
                    let new_sym = SymTab::new(sym.clone(), &[decl.var.clone()]);
                    let stmts = try!(self.analyze_stmts(&Rc::new(new_sym), iter, st));
                    let block = exec::Block::new(decl.loc.clone(), true, val, stmts);
                    ret.push(exec::Statement::Block(block));
                    break;
                }
                
                _ => ret.push(try!(stmt.analyze(sym, st))),
            }
        }
        
        Ok(ret)
    }

    // TODO: optimize this
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::Block> {
        //println!("Block::analyze(): {:?}\n", self);

        let iter = (&self.stmts).iter();
        let stmts = try!(self.analyze_stmts(sym, iter, st));
        Ok(exec::Block::new(self.loc.clone(), false, None, stmts))
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

    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::IfStatement> {
        let test = Box::new(try!(self.test.analyze(sym, st)));
        let true_stmt = Box::new(try!(self.true_stmt.analyze(sym, st)));
        let false_stmt = match self.false_stmt {
            Some(ref f) => Some(Box::new(try!(f.analyze(sym, st)))),
            None => None,
        };
        Ok(exec::IfStatement::new(self.loc.clone(), test, true_stmt, false_stmt))
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

    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::WhileStatement> {
        let test = Box::new(try!(self.test.analyze(sym, st)));
        
        st.save_state();
        try!(st.set_allow_break(true, &self.loc));
        let stmt = Box::new(try!(self.stmt.analyze(sym, st)));
        try!(st.restore_state(&self.loc));
        
        Ok(exec::WhileStatement::new(self.loc.clone(), test, stmt))
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

    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::ReturnStatement> {
        let expr = match self.expr {
            Some(ref e) => Some(Box::new(try!(e.analyze(sym, st)))),
            None => None,
        };
        Ok(exec::ReturnStatement::new(self.loc.clone(), expr))
    }
}

// =========================================================
// VarDecl
pub struct VarDecl {
    pub var : Rc<String>,
    pub val : Option<Box<Expression>>,
    pub loc : SrcLoc,
}

impl VarDecl {
    pub fn new(loc : SrcLoc, var : Rc<String>, val : Option<Box<Expression>>) -> VarDecl {
        VarDecl {
            var : var,
            val : val,
            loc : loc,
        }
    }
}


// =========================================================
// FuncDef
pub struct FuncDef {
    pub params : Vec<Rc<String>>,
    pub block : Box<Block>,
    pub loc : SrcLoc,
}

impl FuncDef {
    pub fn new(loc : SrcLoc, params : Vec<Rc<String>>, block : Box<Block>) -> FuncDef {
        FuncDef {
            params : params,
            block : block,
            loc : loc,
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::FuncDef> {
        //println!("FuncDef::analyze(): {:?}\n", self);
        let new_sym = Rc::new(SymTab::new(sym.clone(), &self.params));
        
        st.save_state();
        try!(st.set_allow_break(false, &self.loc));
        let block = try!(self.block.analyze(&new_sym, st));
        try!(st.restore_state(&self.loc));
        
        Ok(exec::FuncDef::new(self.loc.clone(), self.params.len(), Box::new(block))) 
    }
}

// =========================================================
// NamedFuncDef
pub struct NamedFuncDef {
    pub name : Rc<String>,
    pub def : Rc<FuncDef>,
}

impl NamedFuncDef {
    pub fn new(name : Rc<String>, def : Rc<FuncDef>) -> NamedFuncDef {
        NamedFuncDef {
            name : name,
            def : def
        }
    }
    
    pub fn analyze(&self, sym : &Rc<SymTab>, st : &mut analysis::State) -> ParseResult<exec::FuncDef> {
        //println!("NamedFuncDef::analyze(): {:?}\n", self);
        self.def.analyze(sym, st)
    }
    
}
