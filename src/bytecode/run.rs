use std::rc::Rc;

use super::super::Env;
use super::super::Value;
use super::super::RunError;
use super::super::src_loc::SrcLoc;
use super::opcodes::*;
use super::Closure;
use super::instr;
use super::super::exec;

const INVALID_ADDR : u32 = (-1i32) as u32;

pub struct Run {
    ip : u32,
    env : Rc<Env>,
    env_stack : Vec<Rc<Env>>,
    val_stack : Vec<Value>,
    ret_stack : Vec<u32>,
    flag_true : bool,
}

impl Run {
    pub fn new(env : Rc<Env>) -> Run {
        Run {
            env : env,
            ip : 0,
            env_stack : vec![],
            val_stack : vec![],
            ret_stack : vec![],
            flag_true : false,
        }
    }
    
    pub fn reset(&mut self, env : Rc<Env>) {
        self.env = env;
        self.ip = 0;
        self.env_stack.clear();
        self.val_stack.clear();
        self.ret_stack.clear();
        self.flag_true = false;
    }
    
    pub fn call_func(&mut self, closure : &Closure, args : &[Value], loc : &SrcLoc, instr : &[u32], literals : &[Value]) -> Result<Value, RunError> {
        if closure.num_params != args.len() {
            return Err(RunError::new_script(loc.clone(), &format!("invalid number of arguments passed (expected {}, got {})", closure.num_params, args.len())));
        }
        
        self.env_stack.push(closure.env.clone());  // push dummy env to keep 'ret' happy
        self.env = Rc::new(Env::new(closure.env.clone(), args));
        self.ret_stack.push(INVALID_ADDR);
        self.ip = closure.addr;
        while self.ip != INVALID_ADDR {
            try!(self.exec_instr(1000, instr, literals));
        }

        // sanity checks        
        if self.val_stack.len() != 1 {
            println!("!!!WARNING!!! bytecode ended with {} values on the val stack", self.val_stack.len());
        }
        if self.env_stack.len() != 0 {
            println!("!!!WARNING!!! bytecode ended with {} values on the env stack", self.env_stack.len());
        }
        if self.ret_stack.len() != 0 {
            println!("!!!WARNING!!! bytecode ended with {} values on the ret stack", self.ret_stack.len());
        }
        
        match self.val_stack.pop() {
            Some(ret) => Ok(ret),
            None => Err(RunError::new_script(loc.clone(), "bytecode stopped with no return value on stack")),
        }
    }
    
    fn exec_instr(&mut self, n : usize, instr : &[u32], literals : &[Value]) -> Result<(), RunError> {
        let loc = SrcLoc::new("bytecode", 0, 0);

        for _ in 0..n {
            if self.ip == INVALID_ADDR {
                break;
            }
            
            println!("-> exec instr at {:08x}", self.ip);
            let instr = instr[self.ip as usize];
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
                    self.val_stack.push(literals[index as usize].clone());
                    self.ip += 1;
                }
                
                OP_NEWENV => {
                    println!("newenv");
                    let n_args = instr::d_op_12(instr) as usize;
                    let start = self.val_stack.len() - n_args;
                    let args = self.val_stack.drain(start..).collect::<Vec<Value>>();
                    self.env_stack.push(self.env.clone());
                    self.env = Rc::new(Env::new(self.env.clone(), &args));
                    self.ip += 1;
                }

                OP_POPENV => {
                    println!("popenv");
                    let n_envs = instr::d_op_12(instr) as usize;
                    for _ in 0..n_envs {
                        self.env = match self.env_stack.pop() {
                            Some(e) => e,
                            None => return Err(RunError::new_panic(None, "popenv with empty stack"))
                        }
                    }
                    self.ip += 1;
                }
                
                OP_POPVAL => {
                    println!("popval");
                    let n_vals = instr::d_op_12(instr) as usize;
                    for _ in 0..n_vals {
                        match self.val_stack.pop() {
                            Some(_) => {},
                            None => return Err(RunError::new_panic(None, "popval with empty stack"))
                        }
                    }
                    self.ip += 1;
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
                
                OP_SETVAR => {
                    println!("setvar");
                    let (vi, ei) = instr::d_op_12_12(instr);
                    let val = match self.val_stack.pop() {
                        Some(v) => v,
                        None => {
                            println!("{:?}", self.env);
                            return Err(RunError::new_panic(None, "setvar with empty val stack"))
                        }
                    };
                    match self.env.set_value(vi as usize, ei as usize, val.clone()) {
                        Ok(()) => self.val_stack.push(val),
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
                    if self.val_stack.len() < n_args + 1 {
                        return Err(RunError::new_panic(None, "call with not enough values in the val stack"));
                    }
                    //let args = self.val_stack.drain(start..).collect::<Vec<Value>>();
                    let args_pos = self.val_stack.len() - n_args;
                    let func_pos = args_pos - 1;
                    let ret = match self.val_stack[func_pos] {
                        Value::BCClosure(ref c) => {
                            if c.num_params != n_args {
                                return Err(RunError::new_panic(None, &format!("invalid number of arguments passed (expected {}, got {})", c.num_params, n_args)));
                            }
                            self.env_stack.push(self.env.clone());
                            self.env = Rc::new(Env::new(c.env.clone(), &self.val_stack[args_pos..]));
                            self.ret_stack.push(self.ip + 1);
                            self.ip = c.addr;
                            None
                        }
                        
                        Value::NativeFunc(ref f) => {
                            println!("--- calling native function -----------------------");
                            let ret = try!(f.call(&self.val_stack[args_pos..], &self.env, &loc));
                            println!("--- done ------------------------------------------");
                            self.ip += 1;
                            Some(ret)
                        }

                        ref f @ Value::ASTClosure(_) => {
                            self.ip += 1;
                            Some(try!(exec::run_function(f, &self.val_stack[args_pos..], &self.env, &loc)))
                        }
                        
                        _ => return Err(RunError::new_panic(None, &format!("call to invalid function ({})", self.val_stack[func_pos])))
                    };
                    self.val_stack.drain(func_pos..);
                    if let Some(ret) = ret {
                        self.val_stack.push(ret);
                    }
                }
                
                OP_RET => {
                    println!("ret");
                    self.env = match self.env_stack.pop() {
                        Some(e) => e,
                        None => return Err(RunError::new_panic(None, "ret with empty env stack"))
                    };
                    self.ip = match self.ret_stack.pop() {
                        Some(addr) => addr,
                        None => return Err(RunError::new_panic(None, "ret with empty ret stack"))
                    };
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
