use placenew::place_into;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::process::id;
use derive_builder::Builder;
use placenew::place_boxed;




//only 1 thread can use a context at a time
#[derive(Builder)]
pub struct IntEvalContext {
    #[builder(default, setter(custom))]
    slotmap: HashMap<String, u32>,
    #[builder(default, setter(custom))]
    vars: Vec<i64>,
    #[builder(default, setter(custom))]

    function_map: HashMap<String, u32>,
    #[builder(default, setter(custom))]
    functions: Vec<fn(&[i64]) -> i64>,

    #[builder(default, setter(skip))]
    vm: IntVM,
}

impl IntEvalContextBuilder {
    pub fn add_var(&mut self, name: String, def_val: i64) -> &mut Self {
        let vars = self.vars.get_or_insert_with(Vec::new);
        let idx = vars.len() as u32;
        vars.push(def_val);
        
        self.slotmap
            .get_or_insert_with(HashMap::new)
            .insert(name, idx);
        
        self
    }

    pub fn add_function(&mut self, name: String, func: fn(&[i64]) -> i64) -> &mut Self {
        let funcs = self.functions.get_or_insert_with(Vec::new);

        let idx = funcs.len() as u32;
        funcs.push(func);

        self.function_map.get_or_insert_with(HashMap::new)
            .insert(name, idx);

        self
    }
}

pub struct IntVM {
    registers: [i64; 7],
    args: [i64; 12],
    argc: u32,
    stack: Box<[i64; 120]>,
    rsp: u32,
}

impl Default for IntVM {
    fn default() -> Self {
        Self{
            registers: [0; 7],
            args: [0; 12],
            argc: 0,
            stack: place_boxed!([0; 120], [i64; 120]),
            rsp: 0,
        }
    }
}


impl IntEvalContext {

}