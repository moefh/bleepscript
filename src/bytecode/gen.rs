
use std::collections::HashMap;

use super::super::Value;

const OP_NOP     : u8 = 0b11_1111;
const OP_NEWENV  : u8 = 1;
const OP_POPENV  : u8 = 2;
const OP_GETVAR  : u8 = 3;
const OP_SETVAR  : u8 = 4;
const OP_SETELEM : u8 = 5;
const OP_PUSHLIT : u8 = 6;
const OP_TEST    : u8 = 7;
const OP_JMP     : u8 = 8;
const OP_JT      : u8 = 9;
const OP_CALL    : u8 = 10;
const OP_RET     : u8 = 11;

pub type Addr = u32;

pub struct Gen {
    instr : Vec<u32>,
    literals : Vec<Value>,
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
    
    // construct instructions
    
    fn i_op(op : u8) -> u32 {
        (op as u32) << 26
    }
    
    fn i_op_12(op : u8, t : u16) -> u32 {
        ((op as u32) << 26) | ((t as u32) & 0x0fff)
    }

    fn i_op_12_12(op : u8, t1 : u16, t2 : u16) -> u32 {
        ((op as u32) << 26) | (((t1 as u32) & 0x0fff) << 12) | ((t2 as u32) & 0x0fff)
    }

    fn i_op_26(op : u8, t : u32) -> u32 {
        ((op as u32) << 26) | (t & 0x03ff_ffff)
    }

    // de-construct instructions
    
    fn d_op_12(instr : u32) -> u16 {
        (instr & 0x0fff) as u16
    }

    fn d_op_12_12(instr : u32) -> (u16, u16) {
        (((instr >> 12) & 0x0fff) as u16, (instr & 0x0fff) as u16)
    }

    fn d_op_26(instr : u32) -> u32 {
        instr & 0x03ff_ffff
    }
    
    // --------------------
    
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

    pub fn emit_nop(&mut self) -> Addr {
        self.instr.push(Gen::i_op(OP_NOP));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_newenv(&mut self, n_params : u16) -> Addr {
        self.instr.push(Gen::i_op_12(OP_NEWENV, n_params));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_popenv(&mut self, n_envs : u16) -> Addr {
        self.instr.push(Gen::i_op_12(OP_POPENV, n_envs));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_getvar(&mut self, vi : u16, ei : u16) -> Addr {
        self.instr.push(Gen::i_op_12_12(OP_GETVAR, vi, ei));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_setvar(&mut self, vi : u16, ei : u16) -> Addr {
        self.instr.push(Gen::i_op_12_12(OP_SETVAR, vi, ei));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_setelem(&mut self) -> Addr {
        self.instr.push(Gen::i_op(OP_SETELEM));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_pushlit(&mut self, index : usize) -> Addr {
        self.instr.push(Gen::i_op_26(OP_PUSHLIT, index as u32));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_test(&mut self) -> Addr {
        self.instr.push(Gen::i_op(OP_TEST));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_jmp(&mut self, addr : Addr) -> Addr {
        self.instr.push(Gen::i_op_26(OP_JMP, addr));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_jt(&mut self, addr : Addr) -> Addr {
        self.instr.push(Gen::i_op_26(OP_JT, addr));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_call(&mut self, n_args : u16) -> Addr {
        self.instr.push(Gen::i_op_12(OP_CALL, n_args));
        (self.instr.len() - 1) as Addr
    }

    pub fn emit_ret(&mut self) -> Addr {
        self.instr.push(Gen::i_op(OP_RET));
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
                OP_NOP     => print!("nop        "),

                OP_NEWENV  => print!("newenv     {}", Gen::d_op_12(instr)),
                OP_POPENV  => print!("popenv     {}", Gen::d_op_12(instr)),

                OP_GETVAR  => print!("getvar     {}, {}", Gen::d_op_12_12(instr).0, Gen::d_op_12_12(instr).1),
                OP_SETVAR  => print!("setvar     {}, {}", Gen::d_op_12_12(instr).0, Gen::d_op_12_12(instr).1),
                OP_SETELEM => print!("setelem    "),
                OP_PUSHLIT => print!("pushlit    {}", Gen::d_op_26(instr)),

                OP_TEST    => print!("test       "),
                OP_JMP     => print!("jmp        {:08x}", Gen::d_op_26(instr)),
                OP_JT      => print!("jt         {:08x}", Gen::d_op_26(instr)),

                OP_CALL    => print!("call       {}", Gen::d_op_12(instr)),
                OP_RET     => print!("ret        "),
                
                _          => print!("???        "),
            }
            
            if let Some(comment) = self.comments.get(&(addr as u32)) {
                print!("    ; {}", comment);
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
