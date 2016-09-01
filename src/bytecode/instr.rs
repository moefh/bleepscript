
// -------------------------------------
// de-construct instructions

#[inline(always)]
pub fn d_op_12(instr : u32) -> u16 {
    (instr & 0x0fff) as u16
}

#[inline(always)]
pub fn d_op_12_12(instr : u32) -> (u16, u16) {
    (((instr >> 12) & 0x0fff) as u16, (instr & 0x0fff) as u16)
}

#[inline(always)]
pub fn d_op_26(instr : u32) -> u32 {
    instr & 0x03ff_ffff
}

// -------------------------------------
// construct instructions

pub fn c_op(op : u8) -> u32 {
    (op as u32) << 26
}

pub fn c_op_12(op : u8, t : u16) -> u32 {
    ((op as u32) << 26) | ((t as u32) & 0x0fff)
}

pub fn c_op_12_12(op : u8, t1 : u16, t2 : u16) -> u32 {
    ((op as u32) << 26) | (((t1 as u32) & 0x0fff) << 12) | ((t2 as u32) & 0x0fff)
}

pub fn c_op_26(op : u8, t : u32) -> u32 {
    ((op as u32) << 26) | (t & 0x03ff_ffff)
}

// -------------------------------------
// fix instructions

pub fn _f_op_12(instr : u32, t : u16) -> u32 {
    (instr & 0xfc00_0000) | ((t as u32) & 0x0fff)
}

pub fn f_op_12_12(instr : u32, t1 : u16, t2 : u16) -> u32 {
    (instr & 0xfc00_0000) | (((t1 as u32) & 0x0fff) << 12) | ((t2 as u32) & 0x0fff)
}

pub fn f_op_26(instr : u32, t : u32) -> u32 {
    (instr & 0xfc00_0000) | (t & 0x03ff_ffff)
}
