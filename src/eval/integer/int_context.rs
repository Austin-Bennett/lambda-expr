use placenew::place_into;

use std::collections::HashMap;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{ Mutex};
use placenew::place_boxed;
use crate::eval::bc::PseudoBuilder;
use crate::eval::integer::bc::IOp;

//only 1 thread can use a context at a time
pub struct IntEvalContext {
    pub(crate) state: Pin<Box<Mutex<IntEvalContextState>>>,
    pub(crate) bc_builder: Mutex<PseudoBuilder>,
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

    pub fn build(&mut self) -> IntEvalContext {
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

impl IntExpr for Vec<IOp> {
    fn eval(&self, ctx: &mut IntEvalContextState) -> i64 {
        ctx.eval_bc(self)
    }
}


//can be evaluated directly, also can be used as a intermediate representation of
//an expression to be later converted into a JIT function
pub struct BcIntExpression<'ctx> {
    pub(crate) context:  NonNull<Mutex<IntEvalContextState>>,
    pub(crate) bc: Vec<IOp>,

    pub(crate) __phantom_lt: PhantomData<&'ctx ()>
}

//todo: Jit functions