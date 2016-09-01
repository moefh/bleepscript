
mod closure;
mod gen;
mod run;
mod instr;
mod opcodes;

pub use self::closure::Closure;
pub use self::gen::Program;

use super::Value;
use super::RunError;
use super::src_loc::SrcLoc;

use self::run::Run;

pub type Addr = u32;
pub const INVALID_ADDR : Addr = (-1i32) as Addr;

pub fn run_closure(c : &Closure, args: &[Value], loc : &SrcLoc, program : &mut Program) -> Result<Value, RunError> {
    let mut run = Run::new(c.env.clone());
    run.call_func(c, args, loc, &program.instr, &program.literals)
}
