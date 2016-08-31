
use std;
use std::rc::Rc;

use super::super::exec;
use super::super::bytecode;
use super::super::src_loc::SrcLoc;
use super::super::sym_tab::SymTab;
use super::super::parser::{ParseResult, ParseError};
use super::super::Value;
use super::analysis;
use super::Expression;

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
            Statement::VarDecl(ref d) => Err(ParseError::new(d.loc.clone(), "internal error: trying to parse variable declaration")),
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

    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        match *self {
            Statement::Empty => {
                gen.add_comment("empty statement");
                gen.emit_halt();
            }
            
            Statement::VarDecl(ref d) => {
                return Err(ParseError::new(d.loc.clone(), "internal error: trying to parse variable declaration"));
            }
            
            Statement::Expression(ref e) => {
                try!(e.compile(sym, gen));
                gen.add_comment("statement end");
                gen.emit_popval(1);
            }
            
            Statement::Block(ref b) => try!(b.compile(sym, gen)),

            Statement::If(_) => {
                gen.add_comment("TODO: if");
                gen.emit_halt();
            }
            
            Statement::While(_) => {
                gen.add_comment("TODO: while");
                gen.emit_halt();
            }
            
            Statement::Return(_) => {
                gen.add_comment("TODO: return");
                gen.emit_halt();
            }
            
            Statement::Break(_) => {
                gen.add_comment("TODO: break");
                gen.emit_halt();
            }
        }
        Ok(())
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
            match *stmt {
                Statement::VarDecl(ref decl) => {
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
    
    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        let mut num_envs = 0;
        let mut cur_sym = sym.clone();
        
        for stmt in &self.stmts {
            match *stmt {
                Statement::VarDecl(ref decl) => {
                    match decl.val {
                        Some(ref e) => try!(e.compile(&cur_sym, gen)),
                        None => {
                            let index = gen.add_literal(Value::Null);
                            gen.add_comment("null");
                            gen.emit_pushlit(index);
                        }
                    }
                    cur_sym = Rc::new(SymTab::new(cur_sym.clone(), &[decl.var.clone()]));
                    num_envs += 1;
                    gen.add_comment(&format!("var {} = ...", &*decl.var));
                    gen.emit_newenv(1);
                }
                
                _ => try!(stmt.compile(&cur_sym, gen)),
            }
        }
        
        if num_envs > 0 {
            gen.emit_popenv(num_envs);
        }
        Ok(())
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
    
    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<(u32,usize)> {
        let addr = gen.addr();
        let new_sym = Rc::new(SymTab::new(sym.clone(), &self.params));

        try!(self.block.compile(&new_sym, gen));

        gen.add_comment("auto return null");
        gen.emit_pushlit(0);
        gen.emit_ret();
        Ok((addr, self.params.len()))
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
    
    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<(u32,usize)> {
        //println!("NamedFuncDef::compile(): function '{}'", self.name);
        self.def.compile(sym, gen)
    }
}
