
pub enum Instr {
    NewEnv(u16),
    PopEnv,
    GetVar(u16, u16),
    SetVar(u16, u16),
    Test,
    Jmp(u32),
    JT(u32),
    Call(u16),
    Ret,
}

pub enum Op {
    NewEnv = 1,
    PopEnv,
    GetVar,
    SetVar,
    Test,
    Jmp,
    JT,
    Call,
    Ret,
}

pub type Addr = u32;

pub struct Gen {
    instr : Vec<u32>,
}

impl Gen {
    pub fn new() -> Gen {
        Gen {
            instr : vec![],
        }
    }
    
    fn i_op(op : u8) -> u32 {
        (op as u32) << 26
    }
    
    fn i_op_12(op : u8, t : u16) -> u32 {
        ((op as u32) << 26) | (t & 0x0fff)
    }

    fn i_op_12_12(op : u8, t1 : u16, t2 : u16) -> u32 {
        ((op as u32) << 26) | ((t1 & 0x0fff) << 12) | (t & 0x0fff)
    }

    fn i_op_12_12(op : u8, t1 : u16, t2 : u16) -> u32 {
        ((op as u32) << 26) | ((t1 & 0x0fff) << 12) | (t & 0x0fff)
    }

    fn i_op_26(op : u8, t : u32) -> u32 {
        ((op as u32) << 26) | (t & 0x001f_ffff)
    }


    pub fn gen_newenv(&mut self, n_params : u16) -> Addr {
        self.instr.push(Gen::i_op_12(Op::NewEnv, n_params));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_popenv(&mut self) -> Addr {
        self.instr.push(Gen::i_op(Op::PopEnv));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_getvar(&mut self, vi : u16, ei : u16) -> Addr {
        self.instr.push(Gen::i_op_12_12(Op::GetVar, vi, ei));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_setvar(&mut self, vi : u16, ei : u16) -> Addr {
        self.instr.push(Gen::i_op_12_12(Op::SetVar, vi, ei));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_test(&mut self) -> Addr {
        self.instr.push(Gen::i_op(Op::Test));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_jmp(&mut self, addr : Addr) -> Addr {
        self.instr.push(Gen::i_op_26(Op::Jmp, addr));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_jt(&mut self, addr : Addr) -> Addr {
        self.instr.push(Gen::i_op_26(Op::JmpTrue, addr));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_call(&mut self, n_args : u16) -> Addr {
        self.instr.push(Gen::i_op_12(Op::Call, n_args));
        (self.instr.len() - 1) as Addr
    }

    pub fn gen_ret(&mut self) -> Addr {
        self.instr.push(Gen::i_op(Op::Ret));
        (self.instr.len() - 1) as Addr
    }

}
