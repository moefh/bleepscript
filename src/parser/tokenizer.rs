
use std::rc::Rc;

use super::token::{Token, Keyword};
use super::ops;
use super::{ParseResult, ParseError};
use super::super::src_loc::SrcLoc;
use super::super::readers::{CharReader, ReadError};

struct TokenizerInput {
    chars : Box<CharReader>,
    loc : SrcLoc,
}

pub struct Tokenizer {
    ops : ops::OpTable,
    inputs : Vec<TokenizerInput>,
    saved : Vec<Token>,
    eof : bool,
    no_loc : SrcLoc,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            ops : ops::OpTable::new(),
            inputs : vec![],
            saved : vec![],
            eof : false,
            no_loc : SrcLoc::new("(no file)", 0, 0),
        }
    }

    pub fn reset(&mut self) {
        self.inputs.clear();
        self.saved.clear();
        self.eof = false;
    }

    pub fn ops(&mut self) -> &mut ops::OpTable {
        &mut self.ops
    }

    pub fn src_loc(&self) -> SrcLoc {
        match self.inputs.last() {
            Some(i) => i.loc.new_at(i.chars.line_num(), i.chars.col_num()),
            None => self.no_loc.clone(),
        }
    }

    pub fn add_input(&mut self, reader : Box<CharReader>, loc : SrcLoc) -> ParseResult<()> {
        let input = TokenizerInput {
            chars : reader,
            loc : loc,
        };
        self.inputs.push(input);
        Ok(())
    }
    
    pub fn unget_token(&mut self, tok : Token) {
        self.saved.push(tok);
    }
    
    fn ungetc(&mut self, ch : char) {
        if let Some(input) = self.inputs.last_mut() {
            input.chars.ungetc(ch)
        }
    }
    
    fn getc(&mut self) -> Option<Result<char, ReadError>> {
        
        loop {
            // try to get a character from the current input
            match self.inputs.last_mut() {
                None => return None,
                Some(input) => {
                    match input.chars.getc() {
                        None => {},
                        Some(val) => return Some(val)
                    }
                }
            }
            
            // Go to next input. We'll never pop the last input so we
            // can return a meaningful SrcLoc in case of unexpected EOF  
            if self.inputs.len() <= 1 {
                self.eof = true;
                return None;
            }
            self.inputs.pop();
        }
        
    }

    pub fn next(&mut self) -> Option<ParseResult<Token>> {
        
        if let Some(tok) = self.saved.pop() {
            return Some(Ok(tok));
        }
        
        loop {
            let loc = self.src_loc();
            let ch = match self.getc() {
                None => return None,
                Some(Err(e)) => return Some(Err(ParseError::from_read(loc, e))),
                Some(Ok(c)) => c, 
            };
            
            match ch {
                // comment
                '#' => {
                    loop {
                        match self.getc() {
                            Some(Ok('\n')) | None => break,
                            Some(Ok(_)) => {}
                            Some(Err(e)) => return Some(Err(ParseError::from_read(self.src_loc(), e))),
                        }
                    }
                }
                
                // white space
                ' ' | '\t' | '\r' | '\n' => {}
                
                // punctuation
                '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' => {
                    return Some(Ok(Token::Punct(ch, loc)));
                }
                
                // number
                '0' ... '9' => {
                    let mut buf = String::new();
                    buf.push(ch);
                    let mut got_point = false;
                    loop {
                        match self.getc() {
                            Some(Ok(c @ '0' ... '9')) => buf.push(c),
                            Some(Ok(c @ '.')) if ! got_point => { got_point = true; buf.push(c); }
                            Some(Ok(c)) => { self.ungetc(c); break; }
                            Some(Err(e)) => return Some(Err(ParseError::from_read(loc, e))),
                            None => break,
                        }
                    }
                    return match buf.parse::<f64>() {
                        Ok(num) => Some(Ok(Token::Number(num, loc))),
                        Err(_) => Some(Err(ParseError::new(loc, &format!("invalid number: '{}'", buf)))),
                    };
                }
                
                // identifier or keyword
                'a' ... 'z' | 'A' ... 'Z' | '_' => {
                    let mut buf = String::new();
                    buf.push(ch);
                    loop {
                        match self.getc() {
                            Some(Ok(c @ 'a' ... 'z')) |
                            Some(Ok(c @ 'A' ... 'Z')) |
                            Some(Ok(c @ '0' ... '9')) |
                            Some(Ok(c @ '_')) => buf.push(c),
                            
                            Some(Ok(c)) => { self.ungetc(c); break; }
                            Some(Err(e)) => return Some(Err(ParseError::from_read(loc, e))),
                            None => break,
                        }
                    }
                    if let Some(keyword) = Keyword::from_ident(&buf) {
                        return Some(Ok(Token::Keyword(keyword, loc)));
                    } else {
                        return Some(Ok(Token::Ident(Rc::new(buf), loc)));
                    }
                }
                
                // string
                '"' => {
                    let mut buf = String::new();
                    loop {
                        match self.getc() {
                            Some(Ok('\\')) => {
                                match self.getc() {
                                    Some(Ok('\\')) => buf.push('\\'),
                                    Some(Ok('t'))  => buf.push('\t'),
                                    Some(Ok('r'))  => buf.push('\r'),
                                    Some(Ok('n'))  => buf.push('\n'),
                                    Some(Ok(c)) => { self.ungetc(c); break; }
                                    Some(Err(e)) => return Some(Err(ParseError::from_read(loc, e))),
                                    None => break,
                                }
                            }
                            Some(Ok('"')) => break,
                            Some(Ok(c)) => buf.push(c),
                            Some(Err(e)) => return Some(Err(ParseError::from_read(loc, e))),
                            None => break,
                        }
                    }
                    return Some(Ok(Token::String(Rc::new(buf), loc)));
                }
                
                // any other char is treated as an operator
                _ => {
                    let mut buf = String::new();
                    buf.push(ch);

                    while let Some(c) = self.getc() {
                        match c {
                            Ok(c) => {
                                buf.push(c);
                                if ! self.ops.is_operator(&buf) {
                                    self.ungetc(c);
                                    buf.pop();
                                    break;
                                }
                            }
                            
                            Err(e) => return Some(Err(ParseError::from_read(loc, e))),
                        }
                    }

                    return Some(Ok(Token::Operator(Rc::new(buf), loc)));
                }
            }
        }
        
    }
}
