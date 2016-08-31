
use std::collections::HashMap;

use super::super::Value;
use super::opcodes::*;
use super::instr;

pub type Addr = u32;

pub struct Gen {
    pub instr : Vec<u32>,
    pub literals : Vec<Value>,
    comments : HashMap<Addr, String>,
}

impl Gen {
    pub fn new() -> Gen {
        Gen {
            instr : vec![],
            literals : vec![],
            comments : HashMap::new(),
        }
    }
    
    pub fn addr(&self) -> Addr {
        self.instr.len() as Addr
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

    pub fn emit_halt(&mut self) -> Addr {
        self.instr.push(instr::i_op_26(OP_HALT, 0x3ff_ffff));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_newenv(&mut self, n_params : u16) -> Addr {
        self.instr.push(instr::i_op_12(OP_NEWENV, n_params));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_popenv(&mut self, n_envs : u16) -> Addr {
        self.instr.push(instr::i_op_12(OP_POPENV, n_envs));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_getvar(&mut self, vi : u16, ei : u16) -> Addr {
        self.instr.push(instr::i_op_12_12(OP_GETVAR, vi, ei));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_setvar(&mut self, vi : u16, ei : u16) -> Addr {
        self.instr.push(instr::i_op_12_12(OP_SETVAR, vi, ei));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_setelem(&mut self) -> Addr {
        self.instr.push(instr::i_op(OP_SETELEM));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_pushlit(&mut self, index : usize) -> Addr {
        self.instr.push(instr::i_op_26(OP_PUSHLIT, index as u32));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_test(&mut self) -> Addr {
        self.instr.push(instr::i_op(OP_TEST));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_jmp(&mut self, addr : Addr) -> Addr {
        self.instr.push(instr::i_op_26(OP_JMP, addr));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_jt(&mut self, addr : Addr) -> Addr {
        self.instr.push(instr::i_op_26(OP_JT, addr));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_call(&mut self, n_args : u16) -> Addr {
        self.instr.push(instr::i_op_12(OP_CALL, n_args));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_ret(&mut self) -> Addr {
        self.instr.push(instr::i_op(OP_RET));
        (self.instr.len() - 1) as Addr
    }

    pub fn disasm(&self, labels : &HashMap<Addr, String>) {
        println!("================================================");
        println!("==== INSTRUCTIONS");
        for (addr, &instr) in self.instr.iter().enumerate() {
            if let Some(label) = labels.get(&(addr as u32)) {
                println!("");
                println!(".{}:", label);
            }
            print!("{:08x}:   {:08x}   ", addr, instr);
            
            match (instr>>26) as u8 {
                OP_HALT    => print!("halt       "),

                OP_NEWENV  => print!("newenv     {}", instr::d_op_12(instr)),
                OP_POPENV  => print!("popenv     {}", instr::d_op_12(instr)),

                OP_GETVAR  => print!("getvar     {}, {}", instr::d_op_12_12(instr).0, instr::d_op_12_12(instr).1),
                OP_SETVAR  => print!("setvar     {}, {}", instr::d_op_12_12(instr).0, instr::d_op_12_12(instr).1),
                OP_SETELEM => print!("setelem    "),
                OP_PUSHLIT => print!("pushlit    {}", instr::d_op_26(instr)),

                OP_TEST    => print!("test       "),
                OP_JMP     => print!("jmp        {:08x}", instr::d_op_26(instr)),
                OP_JT      => print!("jt         {:08x}", instr::d_op_26(instr)),

                OP_CALL    => print!("call       {}", instr::d_op_12(instr)),
                OP_RET     => print!("ret        "),
                
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
