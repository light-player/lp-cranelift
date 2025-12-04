use cranelift_codegen::ir::{types, Block};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::Module;
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct CodegenContext<'a> {
    pub builder: FunctionBuilder<'a>,
    pub module: &'a mut dyn Module,
    pub variables: HashMap<String, Variable>,
    
    // Control flow tracking for break/continue
    pub loop_stack: Vec<LoopContext>,
}

pub struct LoopContext {
    pub continue_target: Block,  // Target for continue (might be header or update block)
    pub exit_block: Block,       // Target for break
}

impl<'a> CodegenContext<'a> {
    pub fn new(builder: FunctionBuilder<'a>, module: &'a mut dyn Module) -> Self {
        Self {
            builder,
            module,
            variables: HashMap::new(),
            loop_stack: Vec::new(),
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

