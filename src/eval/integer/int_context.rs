use placenew::place_into;

use std::collections::HashMap;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Mutex;
#[cfg(feature = "jit-compile")]
use std::sync::atomic::AtomicU32;
#[cfg(feature = "jit-compile")]
use inkwell::builder::Builder;
#[cfg(feature = "jit-compile")]
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
#[cfg(feature = "jit-compile")]
use inkwell::{AddressSpace, OptimizationLevel};
#[cfg(feature = "jit-compile")]
use inkwell::types::{FunctionType, IntType};
#[cfg(feature = "jit-compile")]
use inkwell::values::FunctionValue;
use placenew::place_boxed;
use crate::eval::bc::PseudoBuilder;
use crate::eval::integer::bc::IOp;
#[cfg(feature = "jit-compile")]
use crate::eval::integer::RawJitIntExpr;


/// The central evaluation context for integer expressions.
///
/// Owns the variable/function tables, the bytecode compiler, and (when the
/// `jit-compile` feature is enabled) the LLVM JIT engine. Thread-safe: multiple
/// compiled expressions may be held and evaluated concurrently, each locking the
/// shared state only during execution.
pub struct IntEvalContext {
    pub(crate) state: Pin<Box<Mutex<IntEvalContextState>>>,
    #[cfg(feature = "jit-compile")]
    pub(crate) jit_comp: IntJit,
    pub(crate) bc_builder: Mutex<PseudoBuilder>,
    #[cfg(feature = "jit-compile")]
    pub(crate) jit_func_count: AtomicU32,
}



/// Builder for [`IntEvalContext`].
///
/// Register variables and functions before calling [`build`](Self::build) to
/// produce a fully initialised context ready for expression compilation.
pub struct IntEvalContextBuilder {
    varmap: Option<HashMap<String, u32>>,
    vars: Option<Vec<i64>>,
    function_map: Option<HashMap<String, u32>>,
    functions: Option<Vec<fn(&[i64]) -> i64>>,
}

impl IntEvalContextBuilder {
    /// Creates a new builder with empty variable and function tables.
    pub fn new() -> Self {
        Self{
            varmap: Some(HashMap::new()),
            vars: Some(Vec::new()),
            function_map: Some(HashMap::new()),
            functions: Some(Vec::new()),
        }
    }

    /// Registers a named variable with an initial integer value.
    pub fn add_variable(&mut self, name: String, val: i64) -> &mut Self {
        let vars = self.vars.get_or_insert_with(Vec::new);
        let varmap = self.varmap.get_or_insert_with(HashMap::new);

        let idx = vars.len();

        vars.push(val);

        varmap.insert(name, idx as u32);

        self
    }

    /// Registers a named function callable from expressions.
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

    /// Consumes the builder and produces a ready-to-use [`IntEvalContext`].
    ///
    /// Pins the shared state so compiled expressions can safely hold pointers to it.
    /// When the `jit-compile` feature is enabled, also initialises the LLVM JIT
    /// engine (including the `ipow` global mapping).
    pub fn build(&mut self) -> IntEvalContext {
        #[cfg(feature = "jit-compile")]
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
            #[cfg(feature = "jit-compile")]
            jit_func_count: AtomicU32::default(),
            #[cfg(feature = "jit-compile")]
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



/// The register-based virtual machine used to execute bytecode expressions.
///
/// Holds a fixed-size register file, an argument staging area for function
/// calls, and a 120-slot stack for spilling values during complex expressions.
pub struct IntVM {


    //pub(crate) pc: u32,
    pub(crate) registers: [i64; 7],
    pub(crate) args: [i64; 12],
    pub(crate) argc: u8,
    pub(crate) stack: Box<[i64; 120]>,
    pub(crate) rsp: u32,
}

/// Self-referential LLVM JIT context, managed via `ouroboros`.
///
/// Owns the inkwell `Context`, `ExecutionEngine`, and `Builder`, together with
/// pre-computed LLVM types and function-type signatures for the runtime
/// callbacks used by JIT-compiled expressions.
#[cfg(feature = "jit-compile")]
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



/// A type-erased integer expression whose backend is determined at runtime.
///
/// Wraps any [`IntExpr`] implementor (bytecode or JIT) so both can be stored
/// and passed uniformly without exposing the concrete type to callers.
pub struct DynIntExpression<'ctx> {
    pub(crate) context: NonNull<Mutex<IntEvalContextState>>,
    pub(crate) expr: Box<dyn IntExpr + 'ctx>
}


/// A compiled bytecode expression bound to an [`IntEvalContext`].
///
/// Can be evaluated directly via the interpreter or converted into a
/// [`DynIntExpression`]. The `'ctx` lifetime ensures the expression cannot
/// outlive the context whose variable and function tables it references.
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

/// A JIT-compiled integer expression bound to an [`IntEvalContext`].
///
/// Holds a native function pointer produced by LLVM. Evaluation invokes the
/// compiled code directly, passing runtime callbacks for variable loads and
/// function calls via the context's shared state.
#[cfg(feature = "jit-compile")]
pub struct JitIntExpression<'ctx> {
    //SAFETY: the pointer is pointing to a pinned memory address
    //and the expression is bounded by the 'ctx lifetime,
    //so this pointer is valid as long as the expression is valid
    pub(crate) context: NonNull<Mutex<IntEvalContextState>>,
    pub(crate) jit: JitFunction<'ctx, RawJitIntExpr>,
}

#[cfg(feature = "jit-compile")]
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

    /// Registers a new variable in the context at runtime.
    ///
    /// Expressions compiled before this call cannot reference the new variable;
    /// only expressions compiled afterwards will have access to it.
    pub fn add_variable(&self, name: String, default_val: i64) {
        let mut lock = self.state.lock().unwrap();

        let slot = lock.vars.len();
        lock.vars.push(default_val);

        lock.varmap.insert(name, slot as u32);
    }


    /// Registers a new callable function in the context at runtime.
    ///
    /// Only expressions compiled after this call can invoke the function;
    /// previously compiled expressions retain their original function table.
    pub fn add_function(&self, name: String, function: fn(&[i64]) -> i64) {
        let mut lock = self.state.lock().unwrap();

        let slot = lock.functions.len();
        lock.functions.push(function);

        lock.function_map.insert(name, slot as u32);
    }


    /// Updates the value of an existing variable.
    ///
    /// All compiled expressions that reference this variable will see the new
    /// value on their next evaluation. Panics if the variable does not exist.
    pub fn set_variable(&self, name: impl AsRef<str>, val: i64) {
        let mut lock = self.state.lock().unwrap();

        let Some(slot) = lock.varmap.get(name.as_ref()).copied() else {
            panic!("No variable named {} in context!", name.as_ref());
        };

        lock.vars[slot as usize] = val;
    }
}