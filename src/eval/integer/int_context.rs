use placenew::place_into;

use std::collections::HashMap;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::atomic::AtomicU32;
use inkwell::builder::Builder;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::{AddressSpace, OptimizationLevel};
use inkwell::types::{FunctionType, IntType};
use inkwell::values::FunctionValue;
use placenew::place_boxed;
use crate::eval::bc::PseudoBuilder;
use crate::eval::integer::bc::IOp;
use crate::eval::integer::RawJitIntExpr;


pub struct IntEvalContext {
    pub(crate) state: Pin<Box<Mutex<IntEvalContextState>>>,
    pub(crate) jit_comp: IntJit,
    pub(crate) bc_builder: Mutex<PseudoBuilder>,
    pub(crate) jit_func_count: AtomicU32,
}



pub struct IntEvalContextBuilder {
    varmap: Option<HashMap<String, u32>>,
    vars: Option<Vec<i64>>,
    function_map: Option<HashMap<String, u32>>,
    functions: Option<Vec<fn(&[i64]) -> i64>>,
}

impl IntEvalContextBuilder {
    pub fn new() -> Self {
        Self{
            varmap: Some(HashMap::new()),
            vars: Some(Vec::new()),
            function_map: Some(HashMap::new()),
            functions: Some(Vec::new()),
        }
    }

    pub fn add_variable(&mut self, name: String, val: i64) -> &mut Self {
        let vars = self.vars.get_or_insert_with(Vec::new);
        let varmap = self.varmap.get_or_insert_with(HashMap::new);

        let idx = vars.len();

        vars.push(val);

        varmap.insert(name, idx as u32);

        self
    }

    pub fn add_function(&mut self, name: String, function: fn(&[i64]) -> i64) -> &mut Self {
        let functions = self.functions.get_or_insert_with(Vec::new);
        let function_map = self.function_map.get_or_insert_with(HashMap::new);

        let idx = functions.len();

        functions.push(function);

        function_map.insert(name, idx as u32);

        self
    }

    extern "C" fn  __ipow_jit(base: i64, exp: i64) -> i64 {
        base.pow(exp.max(0) as u32)
    }

    pub fn build(&mut self) -> IntEvalContext {
        //prepare the jit context
        let context = Box::new(inkwell::context::Context::create());

        IntEvalContext{
            state: Box::pin(Mutex::new(
                IntEvalContextState{
                    varmap: self.varmap.take().unwrap(),
                    vars: self.vars.take().unwrap(),
                    function_map: self.function_map.take().unwrap(),
                    functions: self.functions.take().unwrap(),
                    vm: IntVM::default(),
                    __pin: PhantomPinned
                }
            )),
            jit_func_count: AtomicU32::default(),
            jit_comp: IntJit::new(
                context,
                |ctx| {

                    let seed_module = ctx.create_module("seed");
                    let i64_ = ctx.i64_type();

                    let ipow_ir = seed_module.add_function(
                        "ipow",
                        i64_.fn_type(&[i64_.into(), i64_.into()], false),
                        None
                    );

                    let eng = seed_module
                        .create_jit_execution_engine(OptimizationLevel::Default)
                        .unwrap();

                    eng.add_global_mapping(&ipow_ir, Self::__ipow_jit as *const () as usize);

                    eng
                },
                |ctx| {
                    ctx.create_builder()
                },
                |ctx, engine| {
                    let ipow_mod = ctx.create_module("ipow");

                    let i64_ = ctx.i64_type();

                    let ipow_ir = ipow_mod.add_function(
                        "ipow",
                        i64_.fn_type(&[i64_.into(), i64_.into()], false),
                        None
                    );
                    engine.add_global_mapping(&ipow_ir, Self::__ipow_jit as *const () as usize);

                    ipow_ir
                },
                |ctx| {
                    ctx.i64_type()
                },
                |ctx| {
                    ctx.i32_type()
                },
                |ctx| {
                    ctx.i8_type()
                },
                |ctx| {
                    let ptr = ctx.ptr_type(AddressSpace::default());
                    ctx.i64_type().fn_type(
                        &[
                            ptr.into(),
                            ptr.into(),
                            ptr.into(),
                            ptr.into(),
                            ptr.into(),
                        ],
                        false
                    )
                },
                |ctx| {
                    let ptr = ctx.ptr_type(AddressSpace::default());
                    ctx.i64_type().fn_type(
                        &[
                            ptr.into(),
                            ctx.i32_type().into(),
                        ],
                        false
                    )
                },
                |ctx| {
                    let ptr = ctx.ptr_type(AddressSpace::default());
                    ctx.void_type().fn_type(
                        &[
                            ptr.into(),
                            ctx.i32_type().into(),
                            ctx.i64_type().into(),
                        ],
                        false
                    )
                },
                |ctx| {
                    let ptr = ctx.ptr_type(AddressSpace::default());
                    ctx.void_type().fn_type(
                        &[
                            ptr.into(),
                            ctx.i8_type().into(),
                        ],
                        false
                    )
                },
                |ctx| {
                    let ptr = ctx.ptr_type(AddressSpace::default());
                    ctx.i64_type().fn_type(
                        &[
                            ptr.into(),
                            ctx.i32_type().into(),
                        ],
                        false
                    )
                },
            ),
            bc_builder: Mutex::new(PseudoBuilder::new()),
        }
    }
}

pub(crate) struct IntEvalContextState {
    pub(crate) varmap: HashMap<String, u32>,
    pub(crate) vars: Vec<i64>,
    pub(crate) function_map: HashMap<String, u32>,
    pub(crate) functions: Vec<fn(&[i64]) -> i64>,
    pub(crate) vm: IntVM,
    
    

    __pin: PhantomPinned
}



pub struct IntVM {


    //pub(crate) pc: u32,
    pub(crate) registers: [i64; 7],
    pub(crate) args: [i64; 12],
    pub(crate) argc: u8,
    pub(crate) stack: Box<[i64; 120]>,
    pub(crate) rsp: u32,
}

#[ouroboros::self_referencing]
pub struct IntJit {
    pub(crate) context: Box<inkwell::context::Context>,

    #[borrows(context)]
    #[covariant]
    pub(crate) engine: ExecutionEngine<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) builder: Builder<'this>,


    #[borrows(context, engine)]
    #[covariant]
    pub(crate) ipow: FunctionValue<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_i64: IntType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_u32: IntType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_u8: IntType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_expr_fn: FunctionType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_load_cb: FunctionType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_store_a_cb: FunctionType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_set_ac_cb: FunctionType<'this>,

    #[borrows(context)]
    #[covariant]
    pub(crate) iw_call_cb: FunctionType<'this>,
}

impl Default for IntVM {
    fn default() -> Self {
        Self{
            //pc: 0,
            registers: [0; 7],
            args: [0; 12],
            argc: 0,
            stack: place_boxed!([0; 120], [i64; 120]),
            rsp: 0,
        }
    }
}


pub(crate) trait IntExpr {
    fn eval(&self, ctx: &mut IntEvalContextState) -> i64;
}



pub struct DynIntExpression<'ctx> {
    pub(crate) context: NonNull<Mutex<IntEvalContextState>>,
    pub(crate) expr: Box<dyn IntExpr + 'ctx>
}


//can be evaluated directly, also can be used as a intermediate representation of
//an expression to be later converted into a JIT function
pub struct BcIntExpression<'ctx> {
    //SAFETY: the pointer is pointing to a pinned memory address
    //and the expression is bounded by the 'ctx lifetime,
    //so this pointer is valid as long as the expression is valid
    pub(crate) context: NonNull<Mutex<IntEvalContextState>>,
    pub(crate) bc: Vec<IOp>,

    pub(crate) __phantom_lt: PhantomData<&'ctx ()>
}

impl IntExpr for Vec<IOp> {
    fn eval(&self, ctx: &mut IntEvalContextState) -> i64 {
        ctx.eval_bc(self)
    }
}

pub struct JitIntExpression<'ctx> {
    //SAFETY: the pointer is pointing to a pinned memory address
    //and the expression is bounded by the 'ctx lifetime,
    //so this pointer is valid as long as the expression is valid
    pub(crate) context: NonNull<Mutex<IntEvalContextState>>,
    pub(crate) jit: JitFunction<'ctx, RawJitIntExpr>,
}

impl<'ctx> IntExpr for JitFunction<'ctx, RawJitIntExpr> {
    fn eval(&self, ctx: &mut IntEvalContextState) -> i64 {
        unsafe{
            self.call(
                ctx as *mut _,
                IntEvalContextState::load_var_callback,
                IntEvalContextState::store_arg_callback,
                IntEvalContextState::set_argc_callback,
                IntEvalContextState::call_function_callback
            )
        }
    }
}


impl IntEvalContext {

    /// adds a new variable for expressions to use,
    /// previous expressions cannot use the new variable (duh)
    pub fn add_variable(&self, name: String, default_val: i64) {
        let mut lock = self.state.lock().unwrap();

        let slot = lock.vars.len();
        lock.vars.push(default_val);

        lock.varmap.insert(name, slot as u32);
    }


    /// adds a new function for expressions to use,
    /// previous expressions cannot use the new function (duh)
    pub fn add_function(&self, name: String, function: fn(&[i64]) -> i64) {
        let mut lock = self.state.lock().unwrap();

        let slot = lock.functions.len();
        lock.functions.push(function);

        lock.function_map.insert(name, slot as u32);
    }


    /// panics if the variable does not exist
    pub fn set_variable(&self, name: impl AsRef<str>, val: i64) {
        let mut lock = self.state.lock().unwrap();

        let Some(slot) = lock.varmap.get(name.as_ref()).copied() else {
            panic!("No variable named {} in context!", name.as_ref());
        };

        lock.vars[slot as usize] = val;
    }
}