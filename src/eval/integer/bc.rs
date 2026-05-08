use crate::eval::bc::{Register};


#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum IOp {
    Load(u32)                   = 0, //loads a VALUE into RET
    PushR(Register)             = 1, //push a register to the stack
    PushV(i64)                  = 2, //push a value to the stack
    Pop(Register)               = 3, //pops a value off the stack into the register
    MoveR(Register, Register)   = 4, //moves the second register into the first
    MoveV(Register, i64)        = 5, //move a value into a register
    MoveArg(u8, i64)            = 6, //move a value into an argument slot
    MoveRArg(u8, Register)      = 7, //move a register into an argument slot
    SetArgCount(u8)             = 8, //set the argument count
    Add(Register, Register)     = 9, //add the second to the first
    Sub(Register, Register)     = 10, //sub
    Mul(Register, Register)     = 11, //mul
    Div(Register, Register)     = 12, //div
    Pow(Register, Register)     = 13, //raises the first to the second
    Neg(Register)               = 14, //negate a register
    Call(u32)                   = 15, //call a routine
}