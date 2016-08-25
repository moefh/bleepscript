
mod char_reader;
mod token;
mod tokenizer;
pub mod ops;
mod errors;

use std;
use std::rc::Rc;

pub use self::errors::ParseError;

use super::SrcLoc;
use self::token::{Token, Keyword};
use super::ast;

pub struct Parser {
    tokens : tokenizer::Tokenizer,
    ops : ops::OpTable,
}

impl Parser {
    pub fn new() -> Parser {
        let mut ops = ops::OpTable::new();
        
        ops.add("+",  70, ops::Assoc::Left);
        ops.add("-",  70, ops::Assoc::Left);
        ops.add("*",  80, ops::Assoc::Left);
        ops.add("/",  80, ops::Assoc::Left);
        ops.add("^", 100, ops::Assoc::Right);

        ops.add("-",  90, ops::Assoc::Prefix);
        
        Parser {
            tokens : tokenizer::Tokenizer::new(),
            ops : ops,
        }
    }

    fn src_loc(&self) -> SrcLoc {
        self.tokens.src_loc()
    }

    fn get_token(&mut self) -> Option<Result<Token, ParseError>> {
        /*
        let tok = self.tokens.next();
        if let Some(Ok(ref tok)) = tok {
            println!("-> {:?}", tok);
        }
        tok
        */
        self.tokens.next()
    }

    fn unget_token(&mut self, tok : Token) {
        self.tokens.unget_token(tok);
    }
    
    fn unexpected_any(&self, loc : SrcLoc, any : &str, expected : &str) -> ParseError {
        ParseError::new(loc, &format!("expected {}, found '{}'", expected, any))
    }
    
    fn unexpected_tok(&self, tok : Token, expected : &str) -> ParseError {
        ParseError::new(tok.peek_loc().clone(), &format!("expected {}, found '{}'", expected, tok))
    }
    
    fn unexpected_eof(&self, expected : &str) -> ParseError {
        ParseError::new(self.src_loc(), &format!("expected {}, found end of file", expected))
    }

    fn expect_punct(&mut self, punct : char) -> Result<SrcLoc, ParseError> {
        match self.get_token() {
            Some(Ok(Token::Punct(ref p, ref loc))) if *p == punct => Ok(loc.clone()),
            Some(Ok(tok)) => Err(self.unexpected_tok(tok, &format!("'{}'", punct))),
            Some(Err(e)) => Err(e),
            None => Err(self.unexpected_eof(&format!("'{}'", punct))),
        }
    }
    
    fn _expect_keyword(&mut self, keyword : Keyword) -> Result<SrcLoc, ParseError> {
        //let loc = self.src_loc();
        match self.get_token() {
            Some(Ok(Token::Keyword(ref k, ref loc))) if *k == keyword => Ok(loc.clone()),
            Some(Ok(tok)) => Err(self.unexpected_tok(tok, keyword.to_ident())),
            Some(Err(e)) => Err(e),
            None => Err(self.unexpected_eof(keyword.to_ident())),
        }
    }

    // ([param [, ...]])
    fn parse_param_list(&mut self) -> Result<Vec<Rc<String>>, ParseError> {

        try!(self.expect_punct('('));
        let mut names = vec![];
        
        // read name or ')'
        match self.get_token() {
            Some(Ok(Token::Punct(')', _))) => return Ok(names),
            Some(Ok(tok @ Token::Ident(..))) => self.unget_token(tok),
            Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "')' or parameter name")),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("')' or parameter name")),
        }
        
        loop {
            // get next name
            match self.get_token() {
                Some(Ok(Token::Ident(name, _))) => names.push(name),
                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "parameter name")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("parameter name")),
            };
            
            // read ',' or ')' 
            match self.get_token() {
                Some(Ok(Token::Punct(',', _))) => {}
                Some(Ok(Token::Punct(')', _))) => return Ok(names),
                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "',' or ')'")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("',' or ')'")),
            }
        }
    }

    // ([expr [, ...]])
    fn parse_arg_list(&mut self) -> Result<Vec<ast::Expression>, ParseError> {

        let mut exprs = vec![];
        
        // check if next is ')'
        match self.get_token() {
            Some(Ok(Token::Punct(')', _))) => return Ok(exprs),
            Some(Ok(tok)) => self.unget_token(tok),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("')' or expression")),
        }
        
        loop {
            // get next expr
            exprs.push(try!(self.parse_expr(false, &[',', ')'])));
            
            // read ',' or ')' 
            match self.get_token() {
                Some(Ok(Token::Punct(',', _))) => {}
                Some(Ok(Token::Punct(')', _))) => return Ok(exprs),
                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "',' or ')'")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("',' or ')'")),
            }
        }
    }

    fn resolve_stack(&mut self, opns : &mut Vec<ast::Expression>, oprs : &mut Vec<(ops::Operator, SrcLoc)>, prec : i32) -> Result<(), ParseError> {
        loop {
            let assoc = if let Some(&(ref opr, _)) = oprs.last() {
                let opr_prec = match opr.assoc {
                    ops::Assoc::Prefix => opr.prec,
                    ops::Assoc::Left => opr.prec,
                    ops::Assoc::Right => opr.prec-1,
                };
                if opr_prec < prec {
                    return Ok(());
                }
                opr.assoc.clone()
            } else {
                return Ok(());
            };
            
            let opn = match assoc {
                ops::Assoc::Left | ops::Assoc::Right => {
                    let (opr, loc) = oprs.pop().unwrap();
                    let right = match opns.pop() {
                        Some(o) => o,
                        None => return Err(ParseError::new(loc, "syntax error")),
                    };
                    let left = match opns.pop() {
                        Some(o) => o,
                        None => return Err(ParseError::new(loc, "syntax error")),
                    };
                    ast::Expression::BinaryOp(ast::BinaryOp::new(loc, opr.name.clone(), Box::new(left), Box::new(right))) 
                }
                
                ops::Assoc::Prefix => {
                    let (opr, loc) = oprs.pop().unwrap();
                    let opn = match opns.pop() {
                        Some(o) => o,
                        None => return Err(ParseError::new(loc, "syntax error")),
                    };
                    ast::Expression::PrefixOp(ast::PrefixOp::new(loc, opr.name.clone(), Box::new(opn)))
                }
            };
            opns.push(opn);
        }
    }

    // expression
    fn parse_expr(&mut self, consume_stop : bool, stop : &[char]) -> Result<ast::Expression, ParseError> {
        
        let mut opns = vec![];
        let mut oprs = vec![];
        let mut expect_opn = true;
        
        loop {
            match self.get_token() {
                Some(Ok(Token::Punct(ref ch, ref loc))) if stop.contains(ch) => {
                    try!(self.resolve_stack(&mut opns, &mut oprs, std::i32::MIN));
                    match opns.len() {
                        0 => return Err(ParseError::new(loc.clone(), "expected expression")),
                        1 => {
                            if ! consume_stop {
                                self.unget_token(Token::Punct(*ch, loc.clone()));
                            }
                            return Ok(opns.pop().unwrap());
                        }
                        _ => return Err(ParseError::new(loc.clone(), "syntax error (stack not empty!)")),
                    }
                }
                
                Some(Ok(Token::Punct('(', loc))) => {
                    if expect_opn {
                        opns.push(try!(self.parse_expr(true, &[')'])));
                    } else {
                        match opns.pop() {
                            Some(func) => opns.push(ast::Expression::FuncCall(ast::FuncCall::new(Box::new(func), try!(self.parse_arg_list())))),
                            None => return Err(ParseError::new(loc, "syntax error (no function on stack!)")),
                        }
                    }
                }

                Some(Ok(Token::Punct('[', loc))) => {
                    if expect_opn {
                        return Err(ParseError::new(loc, "TODO: parse array literal"));
                    } else {
                        return Err(ParseError::new(loc, "TODO: parse array index"));
                    }
                }
                
                Some(Ok(Token::Operator(op, loc))) => {
                    if expect_opn {
                        match self.ops.get_prefix(&op) {
                            Some(op) => {
                                try!(self.resolve_stack(&mut opns, &mut oprs, op.prec));
                                oprs.push((op, loc));
                            },
                            None => return Err(self.unexpected_any(loc, &*op, "expression")),
                        }
                    } else {
                        match self.ops.get_binary(&op) {
                            Some(op) => {
                                try!(self.resolve_stack(&mut opns, &mut oprs, op.prec));
                                oprs.push((op, loc));
                            },
                            None => return Err(self.unexpected_any(loc, &*op, "'(' or operator")),
                        }
                        expect_opn = true;
                    }
                }
                
                Some(Ok(Token::Ident(name, loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::Ident(name, loc));
                        expect_opn = false;
                    } else {
                        return Err(self.unexpected_any(loc, &*name, "'(' or operator"));
                    }
                }
                
                Some(Ok(Token::String(str, loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::String(str, loc));
                        expect_opn = false;
                    } else {
                        return Err(self.unexpected_any(loc, &*str, "'(' or operator"));
                    }
                }
                
                Some(Ok(Token::Number(n, loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::Number(n, loc));
                        expect_opn = false;
                    } else {
                        // function call
                        return Err(ParseError::new(loc, "parsing function call"));
                    }
                }
                
                Some(Ok(Token::Keyword(token::Keyword::Function, loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::FuncDef(try!(self.parse_func_def(loc))))
                    } else {
                        // function call
                        return Err(self.unexpected_any(loc, "function", "'(' or operator"));
                    }
                }

                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "expression")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("expression")),
            }
        }
    }

    // { ... }
    fn parse_block(&mut self) -> Result<ast::Block, ParseError> {
        let block_loc = try!(self.expect_punct('{'));
        
        let mut stmts = vec![];
        
        loop {
            match self.get_token() {
                Some(Ok(Token::Punct('}', _))) => break,
                Some(Ok(tok @ Token::Punct('{', _))) => {
                    self.unget_token(tok);
                    stmts.push(ast::Statement::Block(try!(self.parse_block())));
                },
                
                Some(Ok(tok)) => {
                    self.unget_token(tok);
                    stmts.push(ast::Statement::Expression(try!(self.parse_expr(true, &[';']))));
                }
                
                Some(Err(e)) => return Err(e),
                
                None => return Err(self.unexpected_eof("statement")),
            }
        }
            
        Ok(ast::Block::new(block_loc, stmts))
        //Err(ParseError::new(self.src_loc(), "parsing block"))
    }

    // function (...) { ... }
    fn parse_func_def(&mut self, loc : SrcLoc) -> Result<Rc<ast::FuncDef>, ParseError> {

        let params = try!(self.parse_param_list());
        let block = try!(self.parse_block());

        Ok(Rc::new(ast::FuncDef::new(loc, params, Box::new(block))))
    }
    
    // function name(...) { ... }
    fn parse_named_func_def(&mut self) -> Result<ast::NamedFuncDef, ParseError> {

        let (name, loc) = match self.get_token() {
            Some(Ok(Token::Ident(name, loc))) => (name, loc),
            Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "function name")),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("function name")),
        };
        
        let def = try!(self.parse_func_def(loc));
        
        Ok(ast::NamedFuncDef::new(name, def))
    }
    
    // include "filename"
    fn parse_include(&mut self) -> Result<(), ParseError> {
        
        let (filename, loc) = match self.get_token() {
            Some(Ok(Token::String(str, loc))) => (str, loc),
            Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "string")),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("string")),
        };
        
        self.tokens.add_input(&*filename, Some(loc))
    }
    
    pub fn parse(&mut self, filename : &str) -> Result<Vec<ast::NamedFuncDef>, ParseError> {
        let mut funcs = vec![];
        
        self.tokens.reset();
        try!(self.tokens.add_input(filename, None));
        while let Some(tok) = self.get_token() {
            match try!(tok) {
                Token::Keyword(Keyword::Include, _) => try!(self.parse_include()),
                Token::Keyword(Keyword::Function, _) => funcs.push(try!(self.parse_named_func_def())),
                tok => return Err(self.unexpected_tok(tok, "'include' or 'function'")),
                //tok => println!("-> '{:?}'", tok),
            }
        }
        
        Ok(funcs)
    }
}

