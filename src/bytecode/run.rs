use std::rc::Rc;

use super::super::Env;
use super::super::Value;
use super::super::RunError;
use super::super::src_loc::SrcLoc;
use super::opcodes::*;
use super::Closure;
use super::instr;
use super::super::exec;
use super::super::native;

use super::INVALID_ADDR;

// disable debug:
macro_rules! debugln {
    ($fmt:expr) => {};
    ($fmt:expr, $($args:tt)*) => {};
}

/*
// enable debug:
macro_rules! debugln {
    ($fmt:expr) => {
        println!($fmt)
    };
    
    ($fmt:expr, $($args:tt)*) => {
        println!($fmt, $($args)*)
    };
}
*/

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
    
    pub fn _reset(&mut self, env : Rc<Env>) {
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
        let loc = SrcLoc::new("(bytecode)", 0, 0);
        
        self.env_stack.push(closure.env.clone());  // push dummy env to keep 'ret' happy
        self.env = Rc::new(Env::new(closure.env.clone(), args));
        self.ret_stack.push(INVALID_ADDR);
        self.ip = closure.addr;
        while self.ip != INVALID_ADDR {
            try!(self.exec_instr(100_000, instr, literals, &loc));
        }

        // sanity checks        
        if self.val_stack.len() != 1 {
            debugln!("!!!WARNING!!! bytecode ended with {} values on the val stack", self.val_stack.len());
        }
        if self.env_stack.len() != 0 {
            debugln!("!!!WARNING!!! bytecode ended with {} values on the env stack", self.env_stack.len());
        }
        if self.ret_stack.len() != 0 {
            debugln!("!!!WARNING!!! bytecode ended with {} values on the ret stack", self.ret_stack.len());
        }
        
        match self.val_stack.pop() {
            Some(ret) => Ok(ret),
            None => Err(RunError::new_script(loc.clone(), "bytecode stopped with no return value on stack")),
        }
    }
    
    fn exec_instr(&mut self, n : usize, instr : &[u32], literals : &[Value], loc : &SrcLoc) -> Result<(), RunError> {
        for _ in 0..n {
            if self.ip == INVALID_ADDR {
                break;
            }
            
            debugln!("===========================================");
            debugln!("-> exec instr at {:08x}", self.ip);
            let instr = instr[self.ip as usize];
            debugln!("  -> instr is {:08x}", instr);
            let op = (instr >> 26) as u8;
            debugln!("  -> op is {}", op);
            match op {
                OP_HALT => {
                    debugln!("halt");
                    println!("halting at {:08x}", self.ip);
                    self.ip = INVALID_ADDR;
                    break;
                }
                
                OP_PUSHLIT => {
                    debugln!("pushlit");
                    let index = instr::d_op_26(instr);
                    self.val_stack.push(literals[index as usize].clone());
                    self.ip += 1;
                }
                
                OP_NEWENV => {
                    debugln!("newenv");
                    let (n_args, n_total) = instr::d_op_12_12(instr);
                    let n_args = n_args as usize;
                    let start = self.val_stack.len() - n_args;
                    {
                        let args = &self.val_stack[start..];
                        self.env_stack.push(self.env.clone());
                        self.env = Rc::new(Env::new_partial(self.env.clone(), args, n_total as usize));
                    }
                    self.val_stack.drain(start..);
                    self.ip += 1;
                }

                OP_POPENV => {
                    debugln!("popenv");
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
                    debugln!("popval");
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
                    debugln!("getvar");
                    let (vi, ei) = instr::d_op_12_12(instr);
                    match self.env.get_value(vi as usize, ei as usize) {
                        Ok(v) => self.val_stack.push(v.clone()),
                        Err(e) => {
                            debugln!("{:?}", self.env);
                            return Err(e);
                        }
                    }
                    self.ip += 1;
                }
                
                OP_SETVAR => {
                    debugln!("setvar");
                    let (vi, ei) = instr::d_op_12_12(instr);
                    let val = match self.val_stack.pop() {
                        Some(v) => v,
                        None => {
                            debugln!("{:?}", self.env);
                            return Err(RunError::new_panic(None, "setvar with empty val stack"))
                        }
                    };
                    match self.env.set_value(vi as usize, ei as usize, val.clone()) {
                        Ok(()) => self.val_stack.push(val),
                        Err(e) => {
                            debugln!("{:?}", self.env);
                            return Err(e);
                        }
                    }
                    self.ip += 1;
                }

                OP_GETELEM => {
                    debugln!("getelem");
                    let index = match self.val_stack.pop() {
                        Some(i) => i,
                        None => return Err(RunError::new_panic(None, "getelem with not enough values in the val stack")),
                    };
                    let container = match self.val_stack.pop() {
                        Some(i) => i,
                        None => return Err(RunError::new_panic(None, "getelem with not enough values in the val stack")),
                    };
                    let val = try!(container.get_element(&index));
                    self.val_stack.push(val);
                    self.ip += 1;
                }
                                
                OP_CALL => {
                    debugln!("call");
                    let n_args = instr::d_op_12(instr) as usize;
                    if self.val_stack.len() < n_args + 1 {
                        return Err(RunError::new_panic(None, "call with not enough values in the val stack"));
                    }
                    let func_pos = self.val_stack.len() - (n_args + 1);
                    let args_pos = func_pos + 1;
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
                            debugln!("--- calling native function -----------------------");
                            let ret = try!(f.call(&self.val_stack[args_pos..], &self.env, &loc));
                            debugln!("--- done ------------------------------------------");
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
                    debugln!("ret");
                    self.env = match self.env_stack.pop() {
                        Some(e) => e,
                        None => return Err(RunError::new_panic(None, "ret with empty env stack"))
                    };
                    self.ip = match self.ret_stack.pop() {
                        Some(addr) => addr,
                        None => return Err(RunError::new_panic(None, "ret with empty ret stack"))
                    };
                }
                
                OP_TEST => {
                    debugln!("test");
                    let val = match self.val_stack.pop() {
                        Some(e) => e,
                        None => return Err(RunError::new_panic(None, "test with empty val stack"))
                    };
                    self.flag_true = val.is_true();
                    self.ip += 1;
                }

                OP_JMP => {
                    debugln!("jmp");
                    self.ip = instr::d_op_26(instr);
                }

                OP_JT => {
                    debugln!("jt");
                    if self.flag_true {
                        self.ip = instr::d_op_26(instr);
                    } else {
                        self.ip += 1;
                    }
                }

                OP_JF => {
                    debugln!("jf");
                    if ! self.flag_true {
                        self.ip = instr::d_op_26(instr);
                    } else {
                        self.ip += 1;
                    }
                }
                
                OP_ADD | OP_SUB | OP_MUL | OP_DIV => {
                    debugln!("arithmetic");
                    if self.val_stack.len() < 2 {
                        println!("val stack: {:?}", self.val_stack);
                        return Err(RunError::new_panic(None, "arithmetic op with not enough values in the val stack"));
                    }
                    let args_pos = self.val_stack.len() - 2;
                    let result = match op {
                        OP_ADD => native::func_num_add(&self.val_stack[args_pos..], &self.env),
                        OP_SUB => native::func_num_sub(&self.val_stack[args_pos..], &self.env),
                        OP_MUL => native::func_num_mul(&self.val_stack[args_pos..], &self.env),
                        OP_DIV => native::func_num_div(&self.val_stack[args_pos..], &self.env),
                        _ => return Err(RunError::new_panic(None, "internal error: unhandled arithmetic op")),
                    };
                    self.val_stack.drain(args_pos..);
                    self.val_stack.push(try!(result));
                    self.ip += 1;
                }
                
                _ => {
                    println!("ERROR: unhandled instruction at {:08x}", self.ip);
                    self.ip = INVALID_ADDR;
                    break;
                }
            }
            
            debugln!("val stack: {:?}", self.val_stack);
        }
        Ok(())
    }
    
}
