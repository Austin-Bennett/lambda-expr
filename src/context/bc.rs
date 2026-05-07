use std::mem;
use crate::ast::{BinaryOp, ExprNode, UnaryOp};

#[repr(usize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Register {
    //Expression registers
    Ret = 0,
    Aux = 1,

    //general purpose registers: VALUE
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


//IR that is converted into the appropriate context
#[derive(Clone, Debug)]
pub enum PseudoOp {
    Load(String), //loads a VALUE into RET
    PushR(Register), //push a register to the stack
    PushV(Value), //push a value to the stack
    Pop(Register), //pops a value off the stack into the register
    MoveR(Register, Register), //moves the second register into the first
    MoveV(Register, Value), //move a value into a register
    MoveArg(u32, Value), //move a value into an argument slot
    MoveRArg(u32, Register), //move a register into an argument slot
    SetArgCount(u32),
    Add(Register, Register), //add the second to the first
    Sub(Register, Register), //sub
    Mul(Register, Register), //mul
    Div(Register, Register), //div
    Pow(Register, Register), //raises the first to the second
    Neg(Register),
    Call(String) //call a routine
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
            alloc_stack: Vec::new(),
        }
    }

    pub fn compile_expr(&mut self, node: &ExprNode) -> Vec<PseudoOp> {
        self.register_alloc.fill(false);
        self.alloc_stack.clear();

        let mut output = Vec::new();
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

    fn load(&mut self, output: &mut Vec<PseudoOp>, into: Register) {
        let Some(a) = self.alloc_stack.pop() else {
            panic!("Alloc stack is empty!");
        };

        match a {
            -1 => {
                output.push(PseudoOp::Pop(into));
            }
            0..5 => {
                self.register_alloc[a as usize] = false;
                let greg = Self::get_greg(a as usize);
                if greg != into {
                    output.push(PseudoOp::MoveR(into, greg));
                }
            }

            _ => unreachable!()
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
                output.push(PseudoOp::MoveArg(i as u32, v))
            }
            Data::Reg(r) => {
                output.push(PseudoOp::MoveRArg(i as u32, r))
            }
        }
    }

    fn build_call(&mut self, func: String, args: &Vec<ExprNode>, output: &mut Vec<PseudoOp>) {

        for (i, a) in args.iter().enumerate() {
            let edata = self.__compile(a, output);
            self.load_data_argument(edata, output, i);
        }
        output.push(PseudoOp::SetArgCount(args.len() as u32));
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
                        let r = self.load_data(output, val, Register::Ret);
                        output.push(PseudoOp::Neg(r));

                        Data::Reg(r)
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