use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use anyhow::Error;
use crate::ast::ExprNode;
use crate::eval::bc::PseudoOp;
use crate::eval::integer::bc::IOp;
use crate::eval::integer::DynIntExpression;
use crate::eval::integer::int_context::{BcIntExpression, IntEvalContext, IntEvalContextState};
use crate::lexer::lexer::Tokens;

impl IntEvalContext {


    /// Parses and compiles an expression string into interpreted bytecode.
    ///
    /// Returns a [`BcIntExpression`] that can be evaluated repeatedly or converted
    /// into a [`DynIntExpression`]. Returns an error if the expression contains
    /// unknown identifiers or is syntactically invalid.
    pub fn bytecode_compile(&self, expr: impl AsRef<str>) -> Result<BcIntExpression<'_>, Arc<Error>> {
        let expr = expr.as_ref();
        let expr = Tokens::new(expr);
        let expr = ExprNode::try_from(expr)?;
        let bc = self.bc_builder.lock().unwrap().compile_expr(&expr).map_err(|e| Arc::new(e))?;

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




#[cfg(feature = "jit-compile")]
pub(crate) type LoadVarCallback = extern "C" fn(*mut IntEvalContextState, u32) -> i64;
#[cfg(feature = "jit-compile")]
pub(crate) type StoreArgCallback = extern "C" fn(*mut IntEvalContextState, u32, i64);
#[cfg(feature = "jit-compile")]
pub(crate) type SetArgcCallback = extern "C" fn(*mut IntEvalContextState, u8);
#[cfg(feature = "jit-compile")]
pub(crate) type CallFnCallback = extern "C" fn(*mut IntEvalContextState, u32) -> i64;
#[cfg(feature = "jit-compile")]
pub(crate) type RawJitIntExpr = unsafe extern "C" fn(*mut IntEvalContextState, LoadVarCallback, StoreArgCallback, SetArgcCallback, CallFnCallback) -> i64;

impl<'ctx> BcIntExpression<'ctx> {
    /// Evaluates the expression using the bytecode interpreter.
    ///
    /// Locks the shared context state for the duration of execution, so
    /// concurrent evaluations of different expressions on the same context
    /// are serialised at this point.
    pub fn eval(&self) -> i64 {
        let mut ctx = unsafe{ self.context.as_ref() }.lock().unwrap();

        ctx.eval_bc(&self.bc)
    }

    /// Converts this expression into a type-erased [`DynIntExpression`].
    pub fn to_dynamic(self) -> DynIntExpression<'ctx> {
        DynIntExpression{
            context: self.context,
            expr: Box::new(self.bc),
        }
    }
}

impl<'ctx> DynIntExpression<'ctx> {
    /// Evaluates the expression through its boxed backend (bytecode or JIT).
    pub fn eval(&self) -> i64 {
        let mut ctx = unsafe{ self.context.as_ref() }.lock().unwrap();

        self.expr.eval(&mut ctx)
    }
}