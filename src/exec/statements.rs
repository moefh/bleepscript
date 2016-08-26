
use super::super::SrcLoc;
use super::Expression;

pub enum Statement {
    Block(Block),
    Expression(Expression),
}

// =========================================================
// Block
pub struct Block {
    pub var : Option<(u16, u16, Option<Box<Expression>>)>,
    pub stmts : Vec<Statement>,
    _loc : SrcLoc,
}

impl Block {
    pub fn new(loc : SrcLoc, var : Option<(u16, u16, Option<Box<Expression>>)>, stmts : Vec<Statement>) -> Block {
        Block {
            var : var,
            stmts : stmts,
            _loc : loc,
        }
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
}
