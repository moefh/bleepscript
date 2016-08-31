
pub const OP_HALT    : u8 = 0b11_1111;
pub const OP_NEWENV  : u8 = 1;
pub const OP_POPENV  : u8 = 2;
pub const OP_GETVAR  : u8 = 3;
pub const OP_SETVAR  : u8 = 4;
pub const OP_SETELEM : u8 = 5;
pub const OP_PUSHLIT : u8 = 6;
pub const OP_TEST    : u8 = 7;
pub const OP_JMP     : u8 = 8;
pub const OP_JT      : u8 = 9;
pub const OP_CALL    : u8 = 10;
pub const OP_RET     : u8 = 11;
