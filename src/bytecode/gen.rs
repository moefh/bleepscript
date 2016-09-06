
use std::collections::HashMap;

use super::super::parser::{ParseResult, ParseError};
use super::super::src_loc::SrcLoc;
use super::super::Value;
use super::opcodes::*;
use super::instr;

use super::Addr;

struct FixupContext {
    pub start_env_level : u32,
    pub instr_addrs : Vec<Addr>,
}

impl FixupContext {
    fn new(start_env_level : u32) -> FixupContext {
        FixupContext {
            start_env_level : start_env_level,
            instr_addrs : vec![],
        }
    }
    
    pub fn add(&mut self, instr_addr : Addr) -> ParseResult<()> {
        self.instr_addrs.push(instr_addr);
        Ok(())
    }
    
    pub fn close(self, instr : &mut [u32], fixed_addr : Addr) -> ParseResult<()> {
        for addr in self.instr_addrs {
            instr[addr as usize] = instr::f_op_26(instr[addr as usize], fixed_addr)
        }
        Ok(())
    }
}

pub struct Program {
    pub instr : Vec<u32>,
    pub literals : Vec<Value>,
    while_ctx : Vec<FixupContext>,
    func_ctx : Vec<FixupContext>,
    env_level : u32,
    labels : HashMap<Addr, String>,
    comments : HashMap<Addr, String>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            instr : vec![],
            literals : vec![Value::Null],
            while_ctx : vec![],
            func_ctx : vec![],
            env_level : 0,
            labels : HashMap::new(),
            comments : HashMap::new(),
        }
    }

    pub fn addr(&self) -> Addr {
        self.instr.len() as Addr
    }

    pub fn add_label(&mut self, addr : Addr, comment : &str) {
        self.labels.insert(addr, comment.to_string()); 
    }

    pub fn inc_env_level(&mut self, n : u32) {
        self.env_level += n;
    }
    
    pub fn dec_env_level(&mut self, n : u32) -> ParseResult<()> {
        if self.env_level < n {
            return Err(ParseError::new(SrcLoc::new("(bytecode generation)",0,0), "trying to add fixup without creating context"));
        }
        self.env_level -= n;
        Ok(())
    }
    
    pub fn get_env_level(&mut self) -> u32 {
        self.env_level
    }

    pub fn set_env_level(&mut self, n : u32) {
        self.env_level = n;
    }
    
    pub fn add_comment(&mut self, comment : &str) {
        let addr = self.addr();
        self.comments.insert(addr, comment.to_string()); 
    }

    pub fn add_literal(&mut self, val : Value) -> usize {
        let index = self.literals.len();
        self.literals.push(val);
        index 
    }
    
    pub fn new_func_context(&mut self) {
        let env_level = self.env_level;
        self.func_ctx.push(FixupContext::new(env_level));
    }
    
    pub fn close_func_context(&mut self, fixed_addr : Addr) -> ParseResult<()> {
        match self.func_ctx.pop() {
            Some(ctx) => ctx.close(&mut self.instr, fixed_addr),
            None => Err(ParseError::new(SrcLoc::new("(bytecode generation)",0,0), "no function context to close")),
        }
    }

    pub fn add_return_fixup(&mut self, addr : Addr) -> ParseResult<()> {
        match self.func_ctx.last_mut() {
            None => Err(ParseError::new(SrcLoc::new("(bytecode generation)",0,0), "no context to add return fixup")),
            Some(ctx) => ctx.add(addr),
        }
    }

    pub fn new_while_context(&mut self) {
        let env_level = self.env_level;
        self.while_ctx.push(FixupContext::new(env_level));
    }
    
    pub fn close_while_context(&mut self, fixed_addr : Addr) -> ParseResult<()> {
        match self.while_ctx.pop() {
            Some(ctx) => ctx.close(&mut self.instr, fixed_addr),
            None => Err(ParseError::new(SrcLoc::new("(bytecode generation)",0,0), "no while context to close")),
        }
    }

    pub fn add_break_fixup(&mut self, addr : Addr) -> ParseResult<()> {
        match self.while_ctx.last_mut() {
            None => Err(ParseError::new(SrcLoc::new("(bytecode generation)",0,0), "no context to add break fixup")),
            Some(ctx) => ctx.add(addr),
        }
    }
    
    pub fn get_while_env_level(&self) -> ParseResult<u32> {
        let env_level = self.env_level;
        match self.while_ctx.last() {
            None => Err(ParseError::new(SrcLoc::new("(bytecode generation)",0,0), "not inside while")),
            Some(ctx) => Ok(env_level - ctx.start_env_level),
        }
    }

    pub fn fix_newenv(&mut self, instr_addr : Addr, n_vals : u16, n_total : u16) {
        self.instr[instr_addr as usize] = instr::f_op_12_12(self.instr[instr_addr as usize], n_vals, n_total);
    }

    pub fn fix_jump(&mut self, instr_addr : Addr, target_addr : Addr) {
        self.instr[instr_addr as usize] = instr::f_op_26(self.instr[instr_addr as usize], target_addr);
    }

    pub fn emit_halt(&mut self) {
        self.instr.push(instr::c_op_26(OP_HALT, 0x3ff_ffff));
    }

    pub fn emit_newenv(&mut self, n_params : u16, n_total : u16) {
        self.instr.push(instr::c_op_12_12(OP_NEWENV, n_params, n_total));
    }

    pub fn emit_popenv(&mut self, n_envs : u16) {
        self.instr.push(instr::c_op_12(OP_POPENV, n_envs));
    }

    pub fn emit_getvar(&mut self, vi : u16, ei : u16) {
        self.instr.push(instr::c_op_12_12(OP_GETVAR, vi, ei));
    }

    pub fn emit_setvar(&mut self, vi : u16, ei : u16) {
        self.instr.push(instr::c_op_12_12(OP_SETVAR, vi, ei));
    }

    pub fn emit_getelem(&mut self) {
        self.instr.push(instr::c_op(OP_GETELEM));
    }

    pub fn _emit_setelem(&mut self) {
        self.instr.push(instr::c_op(OP_SETELEM));
    }

    pub fn emit_pushlit(&mut self, index : usize) {
        self.instr.push(instr::c_op_26(OP_PUSHLIT, index as u32));
    }

    pub fn emit_add(&mut self) {
        self.instr.push(instr::c_op(OP_ADD));
    }

    pub fn emit_sub(&mut self) {
        self.instr.push(instr::c_op(OP_SUB));
    }

    pub fn emit_mul(&mut self) {
        self.instr.push(instr::c_op(OP_MUL));
    }

    pub fn emit_div(&mut self) {
        self.instr.push(instr::c_op(OP_DIV));
    }

    pub fn emit_test(&mut self) {
        self.instr.push(instr::c_op(OP_TEST));
    }

    pub fn emit_jmp(&mut self, addr : Addr) {
        self.instr.push(instr::c_op_26(OP_JMP, addr));
    }

    pub fn _emit_jt(&mut self, addr : Addr) {
        self.instr.push(instr::c_op_26(OP_JT, addr));
    }

    pub fn emit_jf(&mut self, addr : Addr) {
        self.instr.push(instr::c_op_26(OP_JF, addr));
    }

    pub fn emit_call(&mut self, n_args : u16) {
        self.instr.push(instr::c_op_12(OP_CALL, n_args));
    }

    pub fn emit_ret(&mut self) {
        self.instr.push(instr::c_op(OP_RET));
    }

    pub fn emit_popval(&mut self, n_vals : u16) {
        self.instr.push(instr::c_op_12(OP_POPVAL, n_vals));
    }

    pub fn disasm(&self) {
        println!("================================================");
        println!("==== INSTRUCTIONS");
        for (addr, &instr) in self.instr.iter().enumerate() {
            if let Some(label) = self.labels.get(&(addr as u32)) {
                println!("");
                println!(".{}:", label);
            }
            print!("{:08x}:   {:08x}   ", addr, instr);
            
            match (instr>>26) as u8 {
                OP_HALT    => print!("halt       "),

                OP_NEWENV  => print!("newenv     {}, {}", instr::d_op_12_12(instr).0, instr::d_op_12_12(instr).1),
                OP_POPENV  => print!("popenv     {}", instr::d_op_12(instr)),

                OP_GETVAR  => print!("getvar     {}, {}", instr::d_op_12_12(instr).0, instr::d_op_12_12(instr).1),
                OP_SETVAR  => print!("setvar     {}, {}", instr::d_op_12_12(instr).0, instr::d_op_12_12(instr).1),

                OP_GETELEM => print!("getelem    "),
                OP_SETELEM => print!("setelem    "),
                OP_PUSHLIT => print!("pushlit    {}", instr::d_op_26(instr)),

                OP_ADD     => print!("add        "),
                OP_SUB     => print!("sub        "),
                OP_MUL     => print!("mul        "),
                OP_DIV     => print!("div        "),

                OP_TEST    => print!("test       "),
                OP_JMP     => print!("jmp        {:08x}", instr::d_op_26(instr)),
                OP_JT      => print!("jt         {:08x}", instr::d_op_26(instr)),
                OP_JF      => print!("jf         {:08x}", instr::d_op_26(instr)),

                OP_CALL    => print!("call       {}", instr::d_op_12(instr)),
                OP_RET     => print!("ret        "),

                OP_POPVAL  => print!("popval     {}", instr::d_op_12(instr)),
                
                _          => print!("???        "),
            }
            
            if let Some(comment) = self.comments.get(&(addr as u32)) {
                print!("      \t; {}", comment);
            }
            println!("");
        }
        
        if self.literals.len() > 0 {
            println!("================================================");
            println!("==== LITERALS");
            for (i, v) in self.literals.iter().enumerate() {
                println!("[{:5} ] {:?}", i, v);
            }
        }
        println!("================================================");
    }
}
