use std::collections::HashMap;
use derive_builder::Builder;

#[derive(Builder)]
pub struct IntEvalContext {
    #[builder(default, setter(custom))]
    slotmap: HashMap<String, u32>,
    #[builder(default, setter(custom))]
    vars: Vec<i64>,
    #[builder(setter(each(name = "function", into)))]
    functions: HashMap<String, fn(*const i64) -> i64>
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
}


impl IntEvalContext {
    
}