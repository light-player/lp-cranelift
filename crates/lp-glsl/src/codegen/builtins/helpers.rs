//! Helper functions for built-in function code generation

use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{types, AbiParam, Signature, FuncRef};
use cranelift_codegen::isa::CallConv;
use cranelift_module::Linkage;

impl<'a> CodegenContext<'a> {
    /// Helper to declare and get FuncRef for external math library function
    pub fn get_math_libcall(&mut self, func_name: &str) -> Result<FuncRef, GlslError> {
        // Create signature: f32 -> f32
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::F32));
        sig.returns.push(AbiParam::new(types::F32));
        
        // Declare function in module if not already declared
        let func_id = self.module
            .declare_function(func_name, Linkage::Import, &sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare external function {}: {}", func_name, e),
                )
            })?;
        
        // Check if the function was actually added to the declarations
        // Some module types (e.g., riscv32 binary compilation) don't support imports
        if !self.module.declarations().functions.is_valid(func_id) {
            // Module doesn't support imports - this is OK for riscv32 with fixed-point
            // The fixed-point transformation will convert the calls anyway
            // Create a local function stub that will be converted by fixed-point transformation
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!("module does not support importing external function {} (this is expected for riscv32 with fixed-point)", func_name),
            ));
        }
        
        // Import into current function
        Ok(self.module.declare_func_in_func(func_id, self.builder.func))
    }

    /// Helper to declare and get FuncRef for atan2 (2-arg function)
    pub fn get_atan2_libcall(&mut self) -> Result<FuncRef, GlslError> {
        // Create signature: (f32, f32) -> f32
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::F32));
        sig.params.push(AbiParam::new(types::F32));
        sig.returns.push(AbiParam::new(types::F32));
        
        // Declare function in module if not already declared
        let func_id = self.module
            .declare_function("atan2f", Linkage::Import, &sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("failed to declare external function atan2f: {}", e),
                )
            })?;
        
        // Import into current function
        Ok(self.module.declare_func_in_func(func_id, self.builder.func))
    }
}

