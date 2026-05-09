use std::mem;
use strum::EnumDiscriminants;
use crate::ast::{BinaryOp, ExprNode, UnaryOp};

#[repr(usize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)] // R1-R5 are constructed via unsafe transmute in get_greg
pub enum Register {
    //Expression registers
    Ret = 0,
    Aux = 1,
    R1 = 2,
    R2 = 3,
    R3 = 4,
    R4 = 5,
    R5 = 6,
}

#[derive(Clone)]
#[derive(Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
}

impl Value {
    pub fn as_i64(&self) -> i64 {
        match self {
            Value::Int(i) => { *i }
            Value::Float(f) => { *f as i64 }
        }
    }

    #[allow(dead_code)]
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Int(i) => { *i as f64 }
            Value::Float(f) => { *f }
        }
    }
}

//IR that is converted into the appropriate context
#[derive(Clone, Debug, EnumDiscriminants)]
#[repr(u8)]
pub enum PseudoOp {
    Load(String)                = 0, //loads a VALUE into RET
    PushR(Register)             = 1, //push a register to the stack
    PushV(Value)                = 2, //push a value to the stack
    Pop(Register)               = 3, //pops a value off the stack into the register
    MoveR(Register, Register)   = 4, //moves the second register into the first
    MoveV(Register, Value)      = 5, //move a value into a register
    MoveArg(u8, Value)          = 6, //move a value into an argument slot
    MoveRArg(u8, Register)      = 7, //move a register into an argument slot
    SetArgCount(u8)             = 8, //set the argument count
    Add(Register, Register)     = 9, //add the second to the first
    Sub(Register, Register)     = 10, //sub
    Mul(Register, Register)     = 11, //mul
    Div(Register, Register)     = 12, //div
    Pow(Register, Register)     = 13, //raises the first to the second
    Neg(Register)               = 14, //negate a register
    Call(String)                = 15, //call a routine
}



pub enum Data {
    Val(Value),
    Reg(Register),
}

pub struct PseudoBuilder {
    //tracks which general registers are currently allocated for some use
    register_alloc: [bool; 5],
    alloc_stack: Vec<i32>,// -1 = allocated to the stack, 0-4 = register
}

impl PseudoBuilder {
    pub fn new() -> Self {
        Self{
            register_alloc: [false; 5],
            //capacity of 10 = nested expression of size 10
            //and makes exponential capacity allocation even better for even more deeply nested functions
            alloc_stack: Vec::with_capacity(10),
        }
    }

    pub fn compile_expr(&mut self, node: &ExprNode) -> Vec<PseudoOp> {
        self.register_alloc.fill(false);
        self.alloc_stack.clear();

        //most likely an expression will require at least 10 nodes
        let mut output = Vec::with_capacity(10);
        match self.__compile(node, &mut output) {
            Data::Val(v) => {
                output.push(PseudoOp::MoveV(Register::Ret, v));
            }
            Data::Reg(r) => {
                if r != Register::Ret {
                    output.push(PseudoOp::MoveR(Register::Ret, r));
                }
            }
        }

        output
    }

    //gets the general register 0-indexed from R1 to R5
    fn get_greg(ind: usize) -> Register {
        if ind >= 5 {
            panic!("{} is not a valid general register!", ind);
        }
        unsafe { mem::transmute(2 + ind) }
    }

    //allocate a register/stack to save a value to
    fn save(&mut self, d: Data, output: &mut Vec<PseudoOp>) {
        //try to save to register
        let mut alloc = -1;

        for (i, r) in self.register_alloc.iter_mut().enumerate() {
            if !*r {
                *r = true;
                alloc = i as i32;
                break;
            }
        }

        self.alloc_stack.push(alloc);

        if alloc == -1 {
            match d {
                Data::Val(v) => {
                    output.push(PseudoOp::PushV(v));
                }
                Data::Reg(r) => {
                    output.push(PseudoOp::PushR(r));
                }
            }
        } else {
            let greg = Self::get_greg(alloc as usize);

            match d {
                Data::Val(v) => {
                    output.push(PseudoOp::MoveV(greg, v));
                }
                Data::Reg(r) => {
                    if greg != r {
                        output.push(PseudoOp::MoveR(greg, r));
                    }
                }
            }
        }
    }

    //returns the register a value is located at, or loads it into the specified register
    fn maybe_load(&mut self, output: &mut Vec<PseudoOp>, into: Register) -> Register {
        let Some(a) = self.alloc_stack.pop() else {
            panic!("Alloc stack is empty!");
        };

        match a {
            -1 => {
                output.push(PseudoOp::Pop(into));
                into
            }
            0..5 => {
                self.register_alloc[a as usize] = false;
                let greg = Self::get_greg(a as usize);

                greg
            }
            _ => unreachable!()
        }
    }


    fn load_data(&mut self, output: &mut Vec<PseudoOp>, d: Data, into: Register) -> Register {
        match d {
            Data::Val(v) => {
                output.push(PseudoOp::MoveV(into, v));
                into
            }
            Data::Reg(r) => {
                r
            }
        }
    }

    fn load_data_argument(&mut self, d: Data, output: &mut Vec<PseudoOp>, i: usize) {
        match d {
            Data::Val(v) => {
                output.push(PseudoOp::MoveArg(i as u8, v))
            }
            Data::Reg(r) => {
                output.push(PseudoOp::MoveRArg(i as u8, r))
            }
        }
    }

    fn build_call(&mut self, func: String, args: &Vec<ExprNode>, output: &mut Vec<PseudoOp>) {

        for (i, a) in args.iter().enumerate() {
            let edata = self.__compile(a, output);
            self.load_data_argument(edata, output, i);
        }
        output.push(PseudoOp::SetArgCount(args.len() as u8));
        output.push(PseudoOp::Call(func));
    }

    fn __compile(&mut self, node: &ExprNode, output: &mut Vec<PseudoOp>) -> Data {
        match node {
            ExprNode::Ident(ident) => {
                output.push(PseudoOp::Load(ident.clone()));

                Data::Reg(Register::Ret)
            }
            ExprNode::Integer(i) => {
                Data::Val(Value::Int(*i))
            }
            ExprNode::Float(f) => {
                Data::Val(Value::Float(*f))
            }
            ExprNode::Binary(bop) => {

                let rhs = self.__compile(&bop.rhs, output);
                self.save(rhs, output);

                let lhs = self.__compile(&bop.lhs, output);

                //loads only if the value isnt already in a register
                let lhs = self.load_data(output, lhs, Register::Ret);
                let rhs = self.maybe_load(output, Register::Aux);

                match bop.operator {
                    BinaryOp::ADD => {
                        output.push(PseudoOp::Add(lhs, rhs));
                    }
                    BinaryOp::SUB => {
                        output.push(PseudoOp::Sub(lhs, rhs));
                    }
                    BinaryOp::MUL => {
                        output.push(PseudoOp::Mul(lhs, rhs));
                    }
                    BinaryOp::DIV => {
                        output.push(PseudoOp::Div(lhs, rhs));
                    }
                    BinaryOp::POW => {
                        output.push(PseudoOp::Pow(lhs, rhs));
                    }
                }

                Data::Reg(lhs)
            }
            ExprNode::Unary(uop) => {
                let val = self.__compile(&uop.node, output);

                match uop.op {
                    UnaryOp::Def => {
                        val
                    }
                    UnaryOp::Neg => {
                        match val {
                            Data::Val(v) => {
                                match v {
                                    Value::Int(i) => { Data::Val(Value::Int(-i)) }
                                    Value::Float(f) => { Data::Val(Value::Float(-f)) }
                                }
                            }
                            Data::Reg(r) => {
                                output.push(PseudoOp::Neg(r));
                                Data::Reg(r)
                            }
                        }
                    }
                }
            }
            ExprNode::Call(call) => {
                match &call.caller {
                    ExprNode::Ident(i) => {
                        //build a call
                        self.build_call(i.clone(), &call.args, output);
                        //a call always returns to RET
                        Data::Reg(Register::Ret)
                    }
                    _ => {
                        unreachable!("caller: {:?}", call.caller);
                    }
                }
            }
        }
    }
}