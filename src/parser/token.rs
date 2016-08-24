
use std::fmt;
use std::rc::Rc;

use super::super::SrcLoc;

#[derive(Clone)]
pub enum Token {
    Keyword(Keyword, SrcLoc),
    Punct(char, SrcLoc),
    Number(f64, SrcLoc),
    String(Rc<String>, SrcLoc),
    Ident(Rc<String>, SrcLoc),
    Operator(Rc<String>, SrcLoc),
}

impl Token {
    pub fn _get_loc(self) -> SrcLoc {
        match self {
            Token::Keyword(_, s) => s,
            Token::Punct(_, s) => s,
            Token::Number(_, s) => s,
            Token::String(_, s) => s,
            Token::Ident(_, s) => s,
            Token::Operator(_, s) => s,
        }
    }

    pub fn peek_loc(&self) -> &SrcLoc {
        match *self {
            Token::Keyword(_, ref s) => s,
            Token::Punct(_, ref s) => s,
            Token::Number(_, ref s) => s,
            Token::String(_, ref s) => s,
            Token::Ident(_, ref s) => s,
            Token::Operator(_, ref s) => s,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Keyword(ref k, _)  => write!(f, "{}", k),
            Token::Punct(ref p, _)    => write!(f, "{}", p),
            Token::Number(ref n, _)   => write!(f, "{}", n),
            Token::String(ref s, _)   => write!(f, "{}", s),
            Token::Ident(ref i, _)    => write!(f, "{}", i),
            Token::Operator(ref o, _) => write!(f, "{}", o),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Keyword(ref k, ref loc)  => write!(f, "keyword[{}]@[{}]", k, loc),
            Token::Punct(ref p, ref loc)    => write!(f, "punct[{}]@[{}]", p, loc),
            Token::Number(ref n, ref loc)   => write!(f, "number[{}]@[{}]", n, loc),
            Token::String(ref s, ref loc)   => write!(f, "string[{:?}]@[{}]", s, loc),
            Token::Ident(ref i, ref loc)    => write!(f, "ident[{}]@[{}]", i, loc),
            Token::Operator(ref o, ref loc) => write!(f, "op[{}]@[{}]", o, loc),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Keyword {
    Include,
    Function,
    Var,
    If,
    Else,
    While,
    Break,
    Return,
}

impl Keyword {
    pub fn to_ident(&self) -> &'static str {
        match *self {
            Keyword::Include  => "include",
            Keyword::Function => "function",
            Keyword::Var      => "var",
            Keyword::If       => "if",
            Keyword::Else     => "else",
            Keyword::While    => "while",
            Keyword::Break    => "break",
            Keyword::Return   => "return",
        }
    }
    
    pub fn from_ident(s : &str) -> Option<Keyword> {
        match s {
            "include"  => Some(Keyword::Include),
            "function" => Some(Keyword::Function),
            "var"      => Some(Keyword::Var),
            "if"       => Some(Keyword::If),
            "else"     => Some(Keyword::Else),
            "while"    => Some(Keyword::While),
            "break"    => Some(Keyword::Break),
            "return"   => Some(Keyword::Return),
            _ => None,
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_ident())
    }
}
