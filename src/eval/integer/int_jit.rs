use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use anyhow::Error;
use inkwell::execution_engine::JitFunction;
use inkwell::values::{IntValue, PointerValue};
use crate::ast::{BinaryOp, ExprNode, UnaryOp};
use crate::eval::integer::{DynIntExpression, IntEvalContext, IntEvalContextState, IntExpr, JitIntExpression, RawJitIntExpr};
use crate::lexer::lexer::Tokens;

impl IntEvalContextState {
    pub(crate) extern "C" fn load_var_callback(this: *mut IntEvalContextState, slot: u32) -> i64 {
        let this = unsafe{ &mut *this };


        //SAFETY only accessed by this crate, so we can guarantee idx won't be out of bounds
        unsafe{ *this.vars.get_unchecked(slot as usize) }
    }

    pub(crate) extern "C" fn store_arg_callback(this: *mut IntEvalContextState, idx: u32, val: i64) {
        let this = unsafe{ &mut *this };


        //SAFETY only accessed by this crate, so we can guarantee idx won't be out of bounds
        *unsafe{ this.vm.args.get_unchecked_mut(idx as usize) } = val;
    }

    pub(crate) extern "C" fn set_argc_callback(this: *mut IntEvalContextState, len: u8) {
        let this = unsafe{ &mut *this };

        this.vm.argc = len;
    }

    pub(crate) extern "C" fn call_function_callback(this: *mut IntEvalContextState, slot: u32) -> i64 {
        let this = unsafe{ &mut *this };


        //SAFETY only accessed by this crate, so we can guarantee idx won't be out of bounds
        let func = unsafe{ *this.functions.get_unchecked(slot as usize) };

        func(&this.vm.args[0..this.vm.argc as usize])
    }
}

impl IntEvalContext {



    pub fn jit_compile(&'_ self, expr: impl AsRef<str>) -> Result<JitIntExpression<'_>, Arc<Error>> {
        let expr = expr.as_ref();
        let expr = Tokens::new(expr);
        let expr = ExprNode::try_from(expr)?;

        let func_name = self.jit_func_count.fetch_add(1, Ordering::Relaxed).to_string();

        let module = self.jit_comp.borrow_context().create_module("expr_mod");
        let func = module.add_function(&func_name, *self.jit_comp.borrow_iw_expr_fn(), None);

        let ctx_0 = func.get_nth_param(0).unwrap().into_pointer_value();
        let load_var_1 = func.get_nth_param(1).unwrap().into_pointer_value();
        let store_arg_2 = func.get_nth_param(2).unwrap().into_pointer_value();
        let set_argc_3 = func.get_nth_param(3).unwrap().into_pointer_value();
        let call_4 = func.get_nth_param(4).unwrap().into_pointer_value();

        let i64_ = *self.jit_comp.borrow_iw_i64();

        let _ipow_ir = module.add_function(
            "ipow",
           i64_.fn_type(&[i64_.into(), i64_.into()], false),
            None
        );



        let block = self.jit_comp.borrow_context().append_basic_block(func, "entry");
        self.jit_comp.borrow_builder().position_at_end(block);

        let mut context = self.state.lock().unwrap();

        let res = self.jit_compile_impl(&expr, &mut context,
            &[ctx_0, load_var_1, store_arg_2, set_argc_3, call_4]
        )?;

        self.jit_comp.with_builder(
            |builder| {
                builder.build_return(Some(&res))
            }
        ).unwrap();

        let func = self.jit_comp.with_engine(
            |engine| {
                engine.add_module(&module).unwrap();

                let func: JitFunction<RawJitIntExpr> = unsafe{
                    engine.get_function(&func_name).unwrap()
                };

                func
            }
        );

        let ctx = self.state.deref() as *const Mutex<IntEvalContextState>;

        Ok(JitIntExpression{
            context: unsafe{ NonNull::new_unchecked(ctx as *mut _) },
            jit: func,
        })
    }



    pub(crate) fn jit_compile_impl<'this>(&'this self,
                                   expr: &ExprNode,
                                   ctx: &mut IntEvalContextState,
                                   args: &[PointerValue<'this>; 5]
    ) -> Result<IntValue<'this>, Arc<Error>> {


        self.jit_comp.with_builder(move |builder| {
            Ok(match expr {
                ExprNode::Ident(identifier) => {
                    let Some(slot) = ctx.varmap.get(identifier).copied() else {
                        return Err(Arc::new(Error::msg(format!("No identifier {} in context!", identifier))));
                    };

                    builder.build_indirect_call(
                        *self.jit_comp.borrow_iw_load_cb(),
                        args[1],
                        &[
                            args[0].into(),
                            self.jit_comp.borrow_iw_u32()
                                .const_int(slot as u64, false)
                                .into()
                        ],
                        "loaded_vm_var"
                    ).unwrap().try_as_basic_value().unwrap_basic().into_int_value()
                }
                ExprNode::Integer(it) => {

                    self.jit_comp.borrow_iw_i64().const_int(it.cast_unsigned(), true)
                }
                ExprNode::Float(f) => {

                    self.jit_comp.borrow_iw_i64().const_int((*f as i64).cast_unsigned(), true)
                }
                ExprNode::Binary(expr) => {
                    let lhs = self.jit_compile_impl(
                        &expr.lhs,
                        ctx,
                        args
                    )?;

                    let rhs = self.jit_compile_impl(
                        &expr.rhs,
                        ctx,
                        args
                    )?;

                    match expr.operator {
                        BinaryOp::ADD => builder.build_int_add(
                                lhs,
                                rhs,
                                "add"
                            ).map_err(|e| Arc::new(e.into()))?,

                        BinaryOp::SUB => builder.build_int_sub(
                            lhs,
                            rhs,
                            "sub"
                        ).map_err(|e| Arc::new(e.into()))?,

                        BinaryOp::MUL => builder.build_int_mul(
                            lhs,
                            rhs,
                            "mul"
                        ).map_err(|e| Arc::new(e.into()))?,

                        BinaryOp::DIV => builder.build_int_signed_div(
                            lhs,
                            rhs,
                            "div"
                        ).map_err(|e| Arc::new(e.into()))?,

                        BinaryOp::POW => builder.build_call(
                            *self.jit_comp.borrow_ipow(),
                            &[lhs.into(), rhs.into()],
                            "pow"
                        ).unwrap().try_as_basic_value().unwrap_basic().into_int_value()
                    }
                }
                ExprNode::Unary(uop) => {
                    let v = self.jit_compile_impl(
                        &uop.node,
                        ctx,
                        args
                    )?;


                    match uop.op {
                        UnaryOp::Def => v,
                        UnaryOp::Neg => builder.build_int_neg(v, "negate").unwrap(),
                    }
                }
                ExprNode::Call(call) => {
                    let slot = match &call.caller {
                        ExprNode::Ident(i) => {
                            let Some(slot) = ctx.function_map.get(i) else {
                                return Err(Arc::new(Error::msg(format!("No identifier {} in context!", i))));
                            };
                            *slot
                        }
                        _ => {
                            unreachable!("caller: {:?}", call.caller);
                        }
                    };

                    builder.build_indirect_call(
                        *self.jit_comp.borrow_iw_set_ac_cb(),
                        args[3],
                        &[
                            args[0].into(),
                            self.jit_comp
                                .borrow_iw_u8()
                                .const_int(call.args.len() as u64, false)
                                .into()
                        ],
                        "set_argc"
                    ).unwrap();

                    for (i, arg) in call.args.iter().enumerate() {
                        let val = self.jit_compile_impl(
                            arg,
                            ctx,
                            args
                        )?;

                        builder.build_indirect_call(
                            *self.jit_comp.borrow_iw_store_a_cb(),
                            args[2].into(),
                            &[
                                args[0].into(),
                                self.jit_comp.borrow_iw_u32()
                                    .const_int(i as u64, false)
                                    .into(),
                                val.into(),
                            ],
                            "store_arg"
                        ).unwrap();
                    }

                    builder.build_indirect_call(
                        *self.jit_comp.borrow_iw_call_cb(),
                        args[4].into(),
                        &[
                            args[0].into(),
                            self.jit_comp.borrow_iw_u32()
                                .const_int(slot as u64, false)
                                .into()
                        ],
                        "call"
                    ).unwrap().try_as_basic_value().unwrap_basic()
                        .into_int_value()
                }
            })

        })
    }
}


impl<'ctx> JitIntExpression<'ctx> {
    pub fn eval(&self) -> i64 {
        let mut ctx = unsafe{ self.context.as_ref() }.lock().unwrap();
        self.jit.eval(ctx.deref_mut())
    }

    pub fn to_dynamic(self) -> DynIntExpression<'ctx> where Self: 'ctx {
        DynIntExpression{
            context: self.context,
            expr: Box::new(self.jit),
        }
    }
}