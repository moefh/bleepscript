
use std::fmt;

use super::*;

trait DebugIndent {
    fn fmt(&self, f : &mut fmt::Formatter, indent : usize) -> Result<(), fmt::Error>;
}

impl DebugIndent for FuncDef {
    fn fmt(&self, f : &mut fmt::Formatter, indent : usize) -> Result<(), fmt::Error> {
        try!(write!(f, "function ("));
        for (n, p) in self.params.iter().enumerate() {
            if n > 0 {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{}", p));
        }
        try!(write!(f, ") "));
        self.block.fmt(f, indent)
    }
}

impl DebugIndent for Expression {
    fn fmt(&self, f : &mut fmt::Formatter, indent : usize) -> Result<(), fmt::Error> {
        match *self {
            Expression::Number(n, _)      => write!(f, "{}", n),
            Expression::String(ref s, _)  => write!(f, "{:?}", **s),
            Expression::Ident(ref i, _)   => write!(f, "{}", **i),
            Expression::FuncDef(ref d)    => d.fmt(f, indent),

            Expression::BinaryOp(ref op) => {
                try!(write!(f, "("));
                try!(op.left.fmt(f, indent));
                try!(write!(f, " {} ", *op.op));
                try!(op.right.fmt(f, indent));
                write!(f, ")")
            }
            
            Expression::PrefixOp(ref op) => {
                try!(write!(f, "("));
                try!(write!(f, "{}", *op.op));
                try!(op.arg.fmt(f, indent));
                write!(f, ")")
            }
            
            Expression::FuncCall(ref c) => {
                try!(c.func.fmt(f, indent));
                try!(write!(f, "("));
                for (n, arg) in c.args.iter().enumerate() {
                    if n > 0 {
                        try!(write!(f, ", "));
                    }
                    try!(arg.fmt(f, indent));
                }
                write!(f, ")")
            }
        }
    }
}

impl DebugIndent for Statement {
    fn fmt(&self, f : &mut fmt::Formatter, indent : usize) -> Result<(), fmt::Error> {
        match *self {
            Statement::Block(ref b)        => b.fmt(f, indent),
            Statement::Expression(ref e)   => e.fmt(f, indent),
        }
    }
}

impl DebugIndent for Block {
    fn fmt(&self, f : &mut fmt::Formatter, indent : usize) -> Result<(), fmt::Error> {
        try!(writeln!(f, "{{"));
        for s in &self.stmts {
            try!(write!(f, "{1:0$}", indent + 2, ""));
            try!(s.fmt(f, indent + 2));
            try!(writeln!(f, ";"));
        }
        write!(f, "{1:0$}}}", indent, "")
    }
}

impl fmt::Debug for NamedFuncDef {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "function {}(", self.name));
        for (n, p) in self.def.params.iter().enumerate() {
            if n > 0 {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{}", p));
        }
        try!(write!(f, ") "));
        self.def.block.fmt(f, 0)
    }
}