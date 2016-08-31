
mod statements;
mod expressions;
mod debug;
mod closure;

pub use self::closure::Closure;
pub use self::statements::*;
pub use self::expressions::*;

use std::rc::Rc;

use super::Env;
use super::Value;
use super::RunError;
use super::src_loc::SrcLoc;

pub fn run_function(func : &Value, args: &[Value], env : &Rc<Env>, loc : &SrcLoc) -> Result<Value, RunError> {
    match *func {
        Value::ASTClosure(ref c) => c.apply(args, &loc),
        Value::BCClosure(_) => Err(RunError::new_script(loc.clone(), "running Bytecode from AST is not implemented!")),
        Value::NativeFunc(ref c) => c.call(args, env, loc),
        ref f => Err(RunError::new_script(loc.clone(), &format!("trying to call non-function value: '{}'", f))),
    }
}
