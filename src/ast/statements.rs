
use std;
use std::rc::Rc;
use super::super::SrcLoc;
use super::Expression;

use super::super::exec;
use super::super::sym_tab::SymTab;
use super::super::parser::ParseResult;
//use super::debug;

pub enum Statement {
    Block(Block),
    VarDecl(VarDecl),
    Expression(Expression),
}

impl Statement {
    pub fn analyze(&self, sym : &Rc<SymTab>) -> ParseResult<exec::Statement> {
        match *self {
            Statement::Block(ref b) => Ok(exec::Statement::Block(try!(b.analyze(sym)))),
            Statement::VarDecl(_) => panic!("trying to analyze variable declaration"),
            Statement::Expression(ref e) => Ok(exec::Statement::Expression(try!(e.analyze(sym)))),
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

    pub fn analyze_stmts<'a>(&self, sym : &Rc<SymTab>, iter : &mut std::slice::Iter<'a, Statement>) -> ParseResult<Vec<exec::Statement>> {
        let mut ret = Vec::new();
        
        while let Some(stmt) = iter.next() {
            match stmt {
                &Statement::VarDecl(ref decl) => {
                    let val = match decl.val {
                        Some(ref e) => Some(Box::new(try!(e.analyze(sym)))),
                        None => None,
                    };
                    let new_sym = Rc::new(SymTab::new(sym.clone(), &[decl.var.clone()]));
                    let stmts = try!(self.analyze_stmts(&new_sym, iter));
                    let block = exec::Block::new(decl.loc.clone(), val, stmts);
                    ret.push(exec::Statement::Block(block));
                    break;
                }
                
                _ => ret.push(try!(stmt.analyze(sym))),
            }
        }
        
        Ok(ret)
    }

    // TODO: optimize this
    pub fn analyze(&self, sym : &Rc<SymTab>) -> ParseResult<exec::Block> {
        //println!("Block::analyze(): {:?}\n", self);

        let mut iter = (&self.stmts).iter();
        let stmts = try!(self.analyze_stmts(sym, &mut iter));
        Ok(exec::Block::new(self.loc.clone(), None, stmts))
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
    
    pub fn analyze(&self, sym : &Rc<SymTab>) -> ParseResult<exec::FuncDef> {
        //println!("FuncDef::analyze(): {:?}\n", self);
        let new_sym = Rc::new(SymTab::new(sym.clone(), &self.params));
        let block = try!(self.block.analyze(&new_sym));
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
    
    pub fn analyze(&self, sym : &Rc<SymTab>) -> ParseResult<exec::FuncDef> {
        //println!("NamedFuncDef::analyze(): {:?}\n", self);
        self.def.analyze(sym)
    }
    
}
