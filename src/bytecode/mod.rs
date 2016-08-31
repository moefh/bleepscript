
mod closure;
mod gen;
mod instr;
mod opcodes;

use std::rc::Rc;

pub use self::closure::Closure;
pub use self::gen::Gen;

use super::Env;
use super::Value;
use super::RunError;
use super::src_loc::SrcLoc;
use self::opcodes::*;

const INVALID_ADDR : u32 = (-1i32) as u32;

pub struct Bytecode {
    instr : Vec<u32>,
    literals : Vec<Value>,
    global_env : Rc<Env>,
    
    ip : u32,
    env : Rc<Env>,
    val_stack : Vec<Value>,
    ret_stack : Vec<u32>,
    flag_true : bool,
}

impl Bytecode {
    pub fn new(global_env : Rc<Env>) -> Bytecode {
        Bytecode {
            global_env : global_env.clone(),

            instr : vec![],
            literals : vec![],
            env : global_env,
            ip : 0,
            val_stack : vec![],
            ret_stack : vec![],
            flag_true : false,
        }
    }
    
    pub fn reset(&mut self, mut gen : Gen) {
        self.instr.clear();
        self.instr.append(&mut gen.instr);
        self.literals.clear();
        self.literals.append(&mut gen.literals);
        
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
        
        self.ip = closure.addr;
        self.env = Rc::new(Env::new(closure.env.clone(), args));
        
        self.ret_stack.push(INVALID_ADDR);
        try!(self.exec_instr(100));
        Ok(Value::Null)
    }
    
    fn exec_instr(&mut self, n : usize) -> Result<(), RunError> {
        for _ in 0..n {
            if self.ip == INVALID_ADDR {
                break;
            }
            
            println!("-> exec instr at {:08x}", self.ip);
            let instr = self.instr[self.ip as usize];
            println!("  -> instr is {:08x}", instr);
            let op = (instr >> 26) as u8;
            println!("  -> op is {}", op);
            match op {
                OP_HALT => {
                    println!("halted at {:08x}\n", self.ip);
                    self.ip = INVALID_ADDR;
                    break;
                }
                
                OP_PUSHLIT => {
                    println!("pushlit");
                    let index = instr::d_op_26(instr);
                    self.val_stack.push(self.literals[index as usize].clone());
                    self.ip += 1;
                }
                
                OP_NEWENV => {
                    println!("newenv");
                    let n_args = instr::d_op_12(instr) as usize;
                    let start = self.val_stack.len() - n_args;
                    let args = self.val_stack.drain(start..).collect::<Vec<Value>>();
                    self.env = Rc::new(Env::new(self.env.clone(), &args));
                    self.ip += 1;
                }

                OP_POPENV => {
                    println!("popenv");
                    let n_envs = instr::d_op_12(instr) as usize;
                    for _ in 0..n_envs {
                        self.env = match self.env.parent {
                            Some(ref e) => e.clone(),
                            None => return Err(RunError::new_panic(None, "popenv with woo many envs"))
                        }
                    }
                    self.ip += 1;
                }
                
                OP_RET => {
                    println!("ret");
                    self.ip = match self.ret_stack.pop() {
                        Some(addr) => addr,
                        None => return Err(RunError::new_panic(None, "ret with empty stack"))
                    };
                }
                
                OP_GETVAR => {
                    println!("getvar");
                    let (vi, ei) = instr::d_op_12_12(instr);
                    match self.env.get_value(vi as usize, ei as usize) {
                        Ok(v) => self.val_stack.push(v.clone()),
                        Err(e) => {
                            println!("{:?}", self.env);
                            return Err(e);
                        }
                    }
                    self.ip += 1;
                }
                
                OP_CALL => {
                    println!("call");
                    let n_args = instr::d_op_12(instr) as usize;
                    let start = self.val_stack.len() - n_args;
                    let args = self.val_stack.drain(start..).collect::<Vec<Value>>();
                    if let Some(func) = self.val_stack.pop() {
                        match func {
                            Value::BCClosure(ref c) => {
                                if c.num_params != n_args {
                                    return Err(RunError::new_panic(None, &format!("invalid number of arguments passed (expected {}, got {})", c.num_params, n_args)));
                                }
                                self.env = Rc::new(Env::new(c.env.clone(), &args));
                                self.ret_stack.push(self.ip);
                                self.ip = c.addr;
                                // TODO: env stack!
                            }
                            
                            Value::NativeFunc(ref f) => {
                                println!("*** calling native function");
                                let loc = SrcLoc::new("bytecode", 0, 0);
                                try!(f.call(&args, &self.env, &loc));
                                self.ip += 1;
                            }
                            
                            _ => return Err(RunError::new_panic(None, &format!("calls to non-bytecode compiled functions not implemented (func={})", func)))
                        }
                    } else {
                        return Err(RunError::new_panic(None, "function call with empty value stack"));
                    }
                }
                
                _ => {
                    println!("unhandled instruction at {:08x}\n", self.ip);
                    self.ip = INVALID_ADDR;
                    break;
                }
            }
        }
        Ok(())
    }
}
