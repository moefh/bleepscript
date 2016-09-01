
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
            Statement::Empty => {}
            
            Statement::VarDecl(ref d) => {
                return Err(ParseError::new(d.loc.clone(), "internal error: trying to parse variable declaration"));
            }
            
            Statement::Expression(ref e) => {
                try!(e.compile(sym, gen));
                gen.add_comment("statement end");
                gen.emit_popval(1);
            }
            
            Statement::Block(ref b) => try!(b.compile(sym, gen)),

            Statement::If(ref i) => try!(i.compile(sym, gen)),
            
            Statement::While(ref w) => try!(w.compile(sym, gen)),
            
            Statement::Return(ref r) => try!(r.compile(sym, gen)),
            
            Statement::Break(ref loc) => {
                // pop envs to match 'while' level
                match gen.get_while_env_level() {
                    Ok(n_envs) => {
                        if n_envs > 0 {
                            gen.emit_popenv(n_envs as u16);
                        }
                    }
                    Err(_) => return Err(ParseError::new(loc.clone(), "'break' not allowed here")),
                }

                // jump to end of while
                let addr = gen.addr();
                gen.add_comment("break");
                gen.emit_jmp(bytecode::INVALID_ADDR);
                try!(gen.add_break_fixup(addr));
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
        let mut cur_sym = sym.clone();
        let mut num_env_vars = 0;
        let mut fix_newenv_addr = 0;
        
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
                    if num_env_vars == 0 {
                        cur_sym = Rc::new(SymTab::new(cur_sym.clone(), &[decl.var.clone()]));
                        gen.inc_env_level(1);
                        fix_newenv_addr = gen.addr();
                        gen.add_comment(&format!("var {} = ...", &*decl.var));
                        gen.emit_newenv(1, 1);
                    } else {
                        let vi = cur_sym.add_name(&*decl.var);
                        gen.add_comment(&format!("var {} = ...", &*decl.var));
                        gen.emit_setvar(vi as u16, 0);
                    }
                    num_env_vars += 1;
                }
                
                _ => try!(stmt.compile(&cur_sym, gen)),
            }
        }
        
        if num_env_vars > 1 {
            gen.fix_newenv(fix_newenv_addr, 1, num_env_vars);
        }
        if num_env_vars > 0 {
            try!(gen.dec_env_level(1));
            gen.emit_popenv(1);
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
    
    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        // test and skip true
        try!(self.test.compile(sym, gen));
        gen.add_comment("if");
        gen.add_comment("if test");
        gen.emit_test();
        let jmp_skip_true = gen.addr();
        gen.add_comment("skip true statement");
        gen.emit_jf(bytecode::INVALID_ADDR);
        
        // true statement
        try!(self.true_stmt.compile(sym, gen));

        if let Some(ref false_stmt) = self.false_stmt {
            let jmp_skip_false = gen.addr();
            gen.add_comment("skip false statement");
            gen.emit_jmp(bytecode::INVALID_ADDR);
            
            let addr_past_true = gen.addr();
            gen.fix_jump(jmp_skip_true, addr_past_true);

            try!(false_stmt.compile(sym, gen));
            let end = gen.addr();
            gen.fix_jump(jmp_skip_false, end);
        } else {
            let addr_past_true = gen.addr();
            gen.fix_jump(jmp_skip_true, addr_past_true);
        }        
        Ok(())
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

    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        gen.new_while_context();

        // test and skip
        let start = gen.addr();
        try!(self.test.compile(sym, gen));
        gen.add_comment("while");
        gen.emit_test();
        let jmp = gen.addr();
        gen.emit_jf(bytecode::INVALID_ADDR);
        
        // statement and jump back
        try!(self.stmt.compile(sym, gen));
        gen.emit_jmp(start);
        let end = gen.addr();
        
        gen.fix_jump(jmp, end);
        try!(gen.close_while_context(end));

        Ok(())
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

    pub fn compile(&self, sym : &Rc<SymTab>, gen : &mut bytecode::Program) -> ParseResult<()> {
        match self.expr {
            Some(ref e) => try!(e.compile(sym, gen)),
            None => gen.emit_pushlit(0),   // null
        }

        gen.add_comment("return");
        let env_level = gen.get_env_level();
        gen.emit_popenv(env_level as u16);
        
        gen.add_comment("return");
        let addr = gen.addr();
        gen.emit_jmp(bytecode::INVALID_ADDR);
        if gen.add_return_fixup(addr).is_err() {
            return Err(ParseError::new(self.loc.clone(), "'return' not allowed here"));
        }
        Ok(())
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
        gen.set_env_level(0);
        gen.new_func_context();

        let addr = gen.addr();
        let new_sym = Rc::new(SymTab::new(sym.clone(), &self.params));

        try!(self.block.compile(&new_sym, gen));

        gen.add_comment("auto return null");
        gen.emit_pushlit(0);
        
        let end = gen.addr();
        gen.emit_ret();

        try!(gen.close_func_context(end));

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
