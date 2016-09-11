
mod token;
mod tokenizer;
pub mod ops;
mod errors;

use std;
use std::rc::Rc;
use std::path;

pub use self::errors::ParseError;

use super::readers::InputSource;
use super::src_loc::SrcLoc;
use self::token::{Token, Keyword};
use super::ast;

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens : tokenizer::Tokenizer,
    base_dir : Option<path::PathBuf>,
    opener : Box<InputSource>,
    fn_call_prec : i32,
    el_index_prec : i32,
}

impl Parser {
    pub fn new(opener : Box<InputSource>) -> Parser {
        Parser {
            tokens : tokenizer::Tokenizer::new(),
            opener : opener,
            base_dir : None,
            fn_call_prec : 0,
            el_index_prec : 0,
        }
    }
    
    pub fn set_function_call_prec(&mut self, prec : i32) {
        self.fn_call_prec = prec;
    }
    
    pub fn set_element_index_prec(&mut self, prec : i32) {
        self.el_index_prec = prec;
    }
    
    pub fn add_op(&mut self, name : &str, prec : i32, assoc : ops::Assoc) {
        self.tokens.ops().add(name, prec, assoc);
    }

    fn ops(&mut self) -> &mut ops::OpTable {
        self.tokens.ops()
    }

    fn src_loc(&self) -> SrcLoc {
        self.tokens.src_loc()
    }

    fn get_token(&mut self) -> Option<ParseResult<Token>> {
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

    fn expect_punct(&mut self, punct : char) -> ParseResult<SrcLoc> {
        match self.get_token() {
            Some(Ok(Token::Punct(p, loc))) => {
                if p == punct {
                    Ok(loc)
                } else {
                    Err(self.unexpected_any(loc, &p.to_string(), &format!("'{}'", punct)))
                }
            }
            Some(Ok(tok)) => Err(self.unexpected_tok(tok, &format!("'{}'", punct))),
            Some(Err(e)) => Err(e),
            None => Err(self.unexpected_eof(&format!("'{}'", punct))),
        }
    }
    
    fn _expect_keyword(&mut self, keyword : Keyword) -> ParseResult<SrcLoc> {
        //let loc = self.src_loc();
        match self.get_token() {
            Some(Ok(Token::Keyword(ref k, ref loc))) if *k == keyword => Ok(loc.clone()),
            Some(Ok(tok)) => Err(self.unexpected_tok(tok, keyword.to_ident())),
            Some(Err(e)) => Err(e),
            None => Err(self.unexpected_eof(keyword.to_ident())),
        }
    }

    // ([param [, ...]])
    fn parse_param_list(&mut self) -> ParseResult<Vec<Rc<String>>> {

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
    fn parse_arg_list(&mut self) -> ParseResult<Vec<ast::Expression>> {

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

    // var ident[ = expr];
    fn parse_var_decl(&mut self) -> ParseResult<ast::VarDecl> {

        let (ident, loc) = match self.get_token() {
            Some(Ok(Token::Ident(ident, loc))) => (ident, loc),
            Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "identifier")),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("identifier")),
        };
        
        // check for '=' 
        let expr = match self.get_token() {
            Some(Ok(Token::Operator(op, loc))) => {
                if *op == "=" {
                    Some(Box::new(try!(self.parse_expr(true, &[';']))))
                } else {
                    return Err(self.unexpected_any(loc, &*op, "'=' or ';'"));
                }
            }
            
            Some(Ok(Token::Punct(';', _))) => None,
            
            Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "'=' or ';'")),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("'=' or ';'")),
        };
        
        Ok(ast::VarDecl::new(loc, ident, expr))
    }

    // [ expr, ... ]
    fn parse_vec_literal(&mut self, loc : SrcLoc) -> ParseResult<ast::VecLiteral> {
        let mut vec : Vec<ast::Expression> = vec![];
        
        loop {
            // read value or ']'
            match self.get_token() {
                Some(Ok(Token::Punct(']', _))) => break,
                Some(Ok(tok)) => {
                    self.unget_token(tok);
                    vec.push(try!(self.parse_expr(false, &[']', ','])));
                }
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("']' or expression")),
            }
            
            // read ']' or ','
            match self.get_token() {
                Some(Ok(Token::Punct(']', _))) => break,
                Some(Ok(Token::Punct(',', _))) => {},
                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "',' or ']'")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("',' or ']'")),
            }
        }
        
        Ok(ast::VecLiteral::new(loc, vec))
    }

    // { ident|string : expr, ... }
    fn parse_map_literal(&mut self, loc : SrcLoc) -> ParseResult<ast::MapLiteral> {
        let mut entries : Vec<(Rc<String>, ast::Expression)> = vec![];
        
        loop {
            // read key: ident or string
            let key = match self.get_token() {
                Some(Ok(Token::Ident(id, _))) => id,
                Some(Ok(Token::String(s, _))) => s,
                Some(Ok(Token::Punct('}', _))) => break,
                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "identifier or string")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("identifier or string")),
            };
            
            try!(self.expect_punct(':'));
            
            // read value: expression
            let val = try!(self.parse_expr(false, &[',', '}']));
            
            // store entry
            entries.push((key, val));

            // read '}' or ','
            match self.get_token() {
                Some(Ok(Token::Punct('}', _))) => break,
                Some(Ok(Token::Punct(',', _))) => {},
                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "',' or '}'")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("',' or '}'")),
            }
        }
        
        Ok(ast::MapLiteral::new(loc, entries))
    }

    fn resolve_stack(&mut self, opns : &mut Vec<ast::Expression>, oprs : &mut Vec<(ops::Operator, SrcLoc)>, prec : i32) -> ParseResult<()> {
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
    fn parse_expr(&mut self, consume_stop : bool, stop : &[char]) -> ParseResult<ast::Expression> {
        
        let mut opns = vec![];
        let mut oprs = vec![];
        let mut expect_opn = true;
        
        loop {
            match self.get_token() {
                Some(Ok(Token::Punct('(', loc))) => {
                    if expect_opn {
                        opns.push(try!(self.parse_expr(true, &[')'])));
                        expect_opn = false;
                    } else {
                        let prec = self.fn_call_prec;
                        try!(self.resolve_stack(&mut opns, &mut oprs, prec));
                        match opns.pop() {
                            Some(func) => {
                                let args = try!(self.parse_arg_list());
                                opns.push(ast::Expression::FuncCall(ast::FuncCall::new(Box::new(func), args)))
                            }
                            None => return Err(ParseError::new(loc, "syntax error (no function on stack!)")),
                        }
                    }
                }

                Some(Ok(Token::Punct('[', loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::Vec(try!(self.parse_vec_literal(loc))));
                        expect_opn = false;
                    } else {
                        let prec = self.el_index_prec;
                        try!(self.resolve_stack(&mut opns, &mut oprs, prec));
                        match opns.pop() {
                            Some(container) => {
                                let index = try!(self.parse_expr(true, &[']']));
                                opns.push(ast::Expression::Element(ast::Element::new(loc, Box::new(container), Box::new(index))))
                            }
                            None => return Err(ParseError::new(loc, "syntax error (no container on stack!)")),
                        }
                    }
                }
                
                Some(Ok(Token::Punct('{', loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::Map(try!(self.parse_map_literal(loc))));
                        expect_opn = false;
                    } else {
                        return Err(self.unexpected_any(loc, "{", "operator"));
                    }
                }
                
                Some(Ok(Token::Punct(ref ch, ref loc))) => {
                    return if stop.contains(ch) {
                        try!(self.resolve_stack(&mut opns, &mut oprs, std::i32::MIN));
                        match opns.len() {
                            0 => Err(ParseError::new(loc.clone(), "expected expression")),
                            1 => {
                                if ! consume_stop {
                                    self.unget_token(Token::Punct(*ch, loc.clone()));
                                }
                                Ok(opns.pop().unwrap())
                            }
                            _ => Err(ParseError::new(loc.clone(), "syntax error (stack not empty!)")),
                        }
                    } else {
                        Err(self.unexpected_any(loc.clone(), &ch.to_string(), "expression"))
                    };
                }
                
                Some(Ok(Token::Operator(op, loc))) => {
                    if expect_opn {
                        match self.ops().get_prefix(&op) {
                            Some(op) => oprs.push((op, loc)),
                            None => return Err(self.unexpected_any(loc, &*op, "expression")),
                        }
                    } else {
                        match self.ops().get_binary(&op) {
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
                        return Err(self.unexpected_any(loc, &n.to_string(), "'(' or operator"));
                    }
                }
                
                Some(Ok(Token::Keyword(token::Keyword::Function, loc))) => {
                    if expect_opn {
                        opns.push(ast::Expression::FuncDef(try!(self.parse_func_def(loc))));
                        expect_opn = false;
                    } else {
                        return Err(self.unexpected_any(loc, "function", "'(' or operator"));
                    }
                }

                Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "expression")),
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("expression")),
            }
        }
    }

    // if (expr) true_stmt; [ else false_stmt; ]
    fn parse_if(&mut self) -> ParseResult<ast::IfStatement> {
        let if_loc = try!(self.expect_punct('('));
        
        let test = Box::new(try!(self.parse_expr(true, &[')'])));
        let true_stmt = Box::new(try!(self.parse_statement()));
        
        let false_stmt = match self.get_token() {
            Some(Ok(Token::Keyword(token::Keyword::Else, _))) => {
                Some(Box::new(try!(self.parse_statement())))
            }
            Some(Ok(tok)) => {
                self.unget_token(tok);
                None
            }
            Some(Err(e)) => return Err(e),
            None => None,
        };
        
        Ok(ast::IfStatement::new(if_loc, test, true_stmt, false_stmt))
    }

    // while (expr) stmt;
    fn parse_while(&mut self) -> ParseResult<ast::WhileStatement> {
        let while_loc = try!(self.expect_punct('('));
        
        let test = Box::new(try!(self.parse_expr(true, &[')'])));
        let stmt = Box::new(try!(self.parse_statement()));
        
        Ok(ast::WhileStatement::new(while_loc, test, stmt))
    }

    // return [expr];
    fn parse_return(&mut self, loc : SrcLoc) -> ParseResult<ast::ReturnStatement> {
        let expr = match self.get_token() {
            Some(Ok(Token::Punct(';', _))) => None,
            Some(Ok(tok)) => {
                self.unget_token(tok);
                Some(Box::new(try!(self.parse_expr(true, &[';']))))
            }
            Some(Err(e)) => return Err(e),
            None => None,
        };
        
        Ok(ast::ReturnStatement::new(loc, expr))
    }

    // any statement
    fn parse_statement(&mut self) -> ParseResult<ast::Statement> {
        match self.get_token() {
            Some(Ok(Token::Punct(';', _))) => Ok(ast::Statement::Empty),
            
            Some(Ok(tok @ Token::Punct('{', _))) => {
                self.unget_token(tok);
                Ok(ast::Statement::Block(try!(self.parse_block())))
            }

            Some(Ok(Token::Keyword(Keyword::Var, _))) => {
                Ok(ast::Statement::VarDecl(try!(self.parse_var_decl())))
            }

            Some(Ok(Token::Keyword(Keyword::If, _))) => {
                Ok(ast::Statement::If(try!(self.parse_if())))
            }
            
            Some(Ok(Token::Keyword(Keyword::While, _))) => {
                Ok(ast::Statement::While(try!(self.parse_while())))
            }
            
            Some(Ok(Token::Keyword(Keyword::Break, loc))) => {
                try!(self.expect_punct(';'));
                Ok(ast::Statement::Break(loc))
            }

            Some(Ok(Token::Keyword(Keyword::Return, loc))) => {
                Ok(ast::Statement::Return(try!(self.parse_return(loc))))
            }

            Some(Ok(tok)) => {
                self.unget_token(tok);
                Ok(ast::Statement::Expression(try!(self.parse_expr(true, &[';']))))
            }
            
            Some(Err(e)) => Err(e),
            
            None => Err(self.unexpected_eof("statement")),
        }
    }

    // { ... }
    fn parse_block(&mut self) -> ParseResult<ast::Block> {
        let block_loc = try!(self.expect_punct('{'));
        
        let mut stmts = vec![];
        loop {
            match self.get_token() {
                Some(Ok(Token::Punct('}', _))) => break,
                Some(Ok(tok)) => {
                    self.unget_token(tok);
                    stmts.push(try!(self.parse_statement()));
                }
                Some(Err(e)) => return Err(e),
                None => return Err(self.unexpected_eof("statement")),
            }
        }
            
        Ok(ast::Block::new(block_loc, stmts))
    }

    // function (...) { ... }
    fn parse_func_def(&mut self, loc : SrcLoc) -> ParseResult<Rc<ast::FuncDef>> {

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
    fn parse_include(&mut self) -> ParseResult<()> {
        
        let (filename, loc) = match self.get_token() {
            Some(Ok(Token::String(str, loc))) => (str, loc),
            Some(Ok(tok)) => return Err(self.unexpected_tok(tok, "string")),
            Some(Err(e)) => return Err(e),
            None => return Err(self.unexpected_eof("string")),
        };
        
        self.add_input(&*filename, Some(loc))
    }
    
    fn parse_script(&mut self) -> ParseResult<Vec<ast::NamedFuncDef>> {
        let mut funcs = vec![];
        
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
    
    fn add_input<P: AsRef<path::Path>>(&mut self, filename : P, loc : Option<SrcLoc>) -> ParseResult<()> {
        let mut path = path::PathBuf::new();
        if let Some(ref dir) = self.base_dir {
            path.push(dir);
        };
        path.push(filename);
        let file_loc = SrcLoc::new(&*path.to_string_lossy(), 0, 0);
        
        let reader = match self.opener.open(&path) {
            Ok(r) => r,
            Err(e) => match loc {
                Some(loc) => return Err(ParseError::from_read(loc, e)),
                None => return Err(ParseError::from_read(file_loc, e)),
            },
        };
        
        self.tokens.add_input(reader, file_loc)
    }

    pub fn parse<P: AsRef<path::Path>>(&mut self, filename : P) -> ParseResult<Vec<ast::NamedFuncDef>> {
        self.tokens.reset();

        let path = filename.as_ref();

        match path.parent() {
            Some(dir) => self.base_dir = Some(path::PathBuf::from(dir)),
            None => self.base_dir = None,
        }

        match path.file_name() {
            Some(file) => try!(self.add_input(file, None)),
            None => return Err(ParseError::new(self.src_loc(), &format!("'{:?}' doesn't specify a file", path))),
        };

        self.parse_script()
    }
}

