
mod closure;

use std::rc::Rc;

pub use self::closure::Closure;

use super::Env;
use super::Value;
use super::RunError;
use super::src_loc::SrcLoc;

struct Bytecode {
    instr : Vec<u32>,
    global_env : Rc<Env>,
    
    ip : u32,
    env : Rc<Env>,
    val_stack : Vec<Value>,
    ret_stack : Vec<u32>,
    flag_true : bool,
}

impl Bytecode {
    pub fn new(instr : Vec<u32>, global_env : Rc<Env>) -> Bytecode {
        Bytecode {
            instr : instr,
            global_env : global_env.clone(),
            
            env : global_env,
            ip : 0,
            val_stack : vec![],
            ret_stack : vec![],
            flag_true : false,
        }
    }
    
    pub fn reset(&mut self) {
        self.env = self.global_env.clone();
        self.ip = 0;
        self.val_stack.clear();
        self.ret_stack.clear();
        self.flag_true = false;
    }
    
    pub fn call_func(&mut self, closure : &Closure, args : &[Value], loc : &SrcLoc) -> Result<Value, RunError> {
        if closure.num_params != args.len() {
            return Err(RunError::new_script(loc.clone(), &format!("invalid number of arguments passed (expected {}, got {})", closure.num_params, args.len())));
        }
        
        self.env = closure.env.clone();
        self.ip = closure.addr;
        
        unimplemented!();
    }
}
