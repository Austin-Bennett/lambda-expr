use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use anyhow::Error;
use crate::ast::ExprNode;
use crate::eval::bc::{PseudoOp, PseudoOpDiscriminants, Value};
use crate::eval::integer::bc::IOp;
use crate::eval::integer::int_context::{BcIntExpression, IntEvalContext, IntEvalContextState, IntExpr};
use crate::lexer::lexer::Tokens;

impl IntEvalContext {


    //turns a string expression into bytecode for quick evaluation and IR storage
    pub fn create_bytecode(&self, expr: impl AsRef<str>) -> Result<BcIntExpression<'_>, Arc<Error>> {
        let expr = expr.as_ref();
        let expr = Tokens::new(expr);
        let expr = ExprNode::try_from(expr)?;
        let bc = self.bc_builder.lock().unwrap().compile_expr(&expr);

        let mut res = Vec::new();

        let state = self.state.lock().unwrap();

        for op in bc {
            let op = match op {
                PseudoOp::Load(s) => {
                    let Some(slot) = state.varmap.get(&s) else {
                        return Err(Arc::new(Error::msg(format!("Unknown identifier {}", s))))
                    };

                    IOp::Load(*slot)
                }
                PseudoOp::PushR(r) => {
                    IOp::PushR(r)
                }
                PseudoOp::PushV(v) => {
                    IOp::PushV(v.as_i64())
                }
                PseudoOp::Pop(r) => {
                    IOp::Pop(r)
                }
                PseudoOp::MoveR(r1, r2) => {
                    IOp::MoveR(r1, r2)
                }
                PseudoOp::MoveV(r1, v) => {
                    IOp::MoveV(r1, v.as_i64())
                }
                PseudoOp::MoveArg(i, v) => {
                    IOp::MoveArg(i, v.as_i64())
                }
                PseudoOp::MoveRArg(i, r) => {
                    IOp::MoveRArg(i, r)
                }
                PseudoOp::SetArgCount(c) => {
                    IOp::SetArgCount(c)
                }
                PseudoOp::Add(r1, r2) => { IOp::Add(r1, r2) }
                PseudoOp::Sub(r1, r2) => { IOp::Sub(r1, r2) }
                PseudoOp::Mul(r1, r2) => { IOp::Mul(r1, r2) }
                PseudoOp::Div(r1, r2) => { IOp::Div(r1, r2) }
                PseudoOp::Pow(r1, r2) => { IOp::Pow(r1, r2) }
                PseudoOp::Neg(r) => { IOp::Neg(r) }
                PseudoOp::Call(func) => {
                    let Some(slot) = state.function_map.get(&func) else {
                        return Err(Arc::new(Error::msg(format!("Unknown identifier: {}", func))));
                    };

                    IOp::Call(*slot)
                }
            };

            res.push(op);
        }

        let ptr = self.state.deref() as *const Mutex<IntEvalContextState>;

        Ok(
            BcIntExpression{
                bc: res,
                //safety: a reference is never null
                context: unsafe{ NonNull::new_unchecked(ptr as *mut Mutex<IntEvalContextState>) },
                __phantom_lt: PhantomData,
            }
        )
    }
}

impl<'ctx> BcIntExpression<'ctx> {
    pub fn eval(&self) -> i64 {
        let mut ctx = unsafe{ self.context.as_ref() }.lock().unwrap();

        self.bc.eval(&mut ctx)
    }
}