use cranelift_codegen::ir::types;
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::Module;
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

pub struct CodegenContext<'a> {
    pub builder: FunctionBuilder<'a>,
    pub module: &'a mut dyn Module,
    pub variables: HashMap<String, Variable>,
}

impl<'a> CodegenContext<'a> {
    pub fn new(builder: FunctionBuilder<'a>, module: &'a mut dyn Module) -> Self {
        Self {
            builder,
            module,
            variables: HashMap::new(),
        }
    }

    pub fn declare_variable(&mut self, name: String, ty: types::Type) -> Variable {
        let var = self.builder.declare_var(ty);
        self.variables.insert(name, var);
        var
    }

    pub fn lookup_variable(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).copied()
    }
}

