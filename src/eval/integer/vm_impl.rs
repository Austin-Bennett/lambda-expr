use crate::eval::bc::Register;
use crate::eval::integer::bc::IOp;
use crate::eval::integer::int_context::{BcIntExpression, IntEvalContextState, IntVM};




impl IntEvalContextState {



    fn vm_push(&mut self, v: i64) {
        self.vm.stack[self.vm.rsp as usize] = v;
        self.vm.rsp += 1;
    }

    fn vm_pop(&mut self) -> i64 {
        let val = self.vm.stack[self.vm.rsp as usize -1];
        self.vm.rsp -= 1;

        val
    }

    pub(crate) fn eval_bc(&mut self, expr: &Vec<IOp>) -> i64 {


        for op in expr {
            match op {
                IOp::Load(slot) => {
                    self.vm.registers[Register::Ret as usize] = self.vars[*slot as usize];
                }
                IOp::PushR(reg) => {
                    self.vm_push(self.vm.registers[*reg as usize]);
                }
                IOp::PushV(val) => {
                    self.vm_push(*val);
                }
                IOp::Pop(reg) => {
                    self.vm.registers[*reg as usize] = self.vm_pop();
                }
                IOp::MoveR(r1, r2) => {
                    self.vm.registers[*r1 as usize] = self.vm.registers[*r2 as usize]
                }
                IOp::MoveV(r1, v) => {
                    self.vm.registers[*r1 as usize] = *v;
                }
                IOp::MoveArg(idx, v) => {
                    self.vm.args[*idx as usize] = *v;
                }
                IOp::MoveRArg(idx, reg) => {
                    self.vm.args[*idx as usize] = self.vm.registers[*reg as usize];
                }
                IOp::SetArgCount(c) => {
                    self.vm.argc = *c;
                }
                IOp::Add(r1, r2) => {
                    self.vm.registers[*r1 as usize] += self.vm.registers[*r2 as usize];
                }
                IOp::Sub(r1, r2) => {
                    self.vm.registers[*r1 as usize] -= self.vm.registers[*r2 as usize];
                }
                IOp::Mul(r1, r2) => {
                    self.vm.registers[*r1 as usize] *= self.vm.registers[*r2 as usize];
                }
                IOp::Div(r1, r2) => {
                    self.vm.registers[*r1 as usize] /= self.vm.registers[*r2 as usize];
                }
                IOp::Pow(r1, r2) => {
                    let r2 = self.vm.registers[*r2 as usize];
                    let r1 = &mut self.vm.registers[*r1 as usize];
                    if r2 < 0 {
                        *r1 = 0;
                    } else {
                        *r1 = r1.pow(r2 as u32);
                    }
                }
                IOp::Neg(reg) => {
                    let r1 = &mut self.vm.registers[*reg as usize];
                    
                    *r1 = -*r1;
                }
                IOp::Call(slot) => {
                    let result = self.functions[*slot as usize](&self.vm.args[0..self.vm.argc as usize]);
                    self.vm.registers[Register::Ret as usize] = result;
                }
            }
        }


        self.vm.registers[Register::Ret as usize]
    }

}