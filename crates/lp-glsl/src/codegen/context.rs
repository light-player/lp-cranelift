use cranelift_codegen::ir::Block;
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::Module;
use hashbrown::HashMap;

use crate::semantic::types::Type as GlslType;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct VarInfo {
    pub cranelift_vars: Vec<Variable>,  // Changed from single Variable to support vectors
    pub glsl_type: GlslType,
}

pub struct CodegenContext<'a> {
    pub builder: FunctionBuilder<'a>,
    pub module: &'a mut dyn Module,
    pub variables: HashMap<String, VarInfo>,
    
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

    pub fn declare_variable(&mut self, name: String, glsl_ty: GlslType) -> Vec<Variable> {
        let component_count = if glsl_ty.is_vector() {
            glsl_ty.component_count().unwrap()
        } else {
            1
        };

        let base_ty = if glsl_ty.is_vector() {
            glsl_ty.vector_base_type().unwrap()
        } else {
            glsl_ty.clone()
        };

        let cranelift_ty = base_ty.to_cranelift_type();
        
        let mut vars = Vec::new();
        for _ in 0..component_count {
            vars.push(self.builder.declare_var(cranelift_ty));
        }

        self.variables.insert(name, VarInfo {
            cranelift_vars: vars.clone(),
            glsl_type: glsl_ty,
        });

        vars
    }

    pub fn lookup_variable(&self, name: &str) -> Option<Variable> {
        // Legacy method: returns first component (for scalars)
        self.variables.get(name).and_then(|info| info.cranelift_vars.first().copied())
    }

    pub fn lookup_variables(&self, name: &str) -> Option<&[Variable]> {
        self.variables.get(name).map(|info| info.cranelift_vars.as_slice())
    }

    pub fn lookup_variable_type(&self, name: &str) -> Option<&GlslType> {
        self.variables.get(name).map(|info| &info.glsl_type)
    }
}

