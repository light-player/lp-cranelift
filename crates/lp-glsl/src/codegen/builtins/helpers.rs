//! Helper functions for built-in function code generation

use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{AbiParam, FuncRef, Signature, types};
use cranelift_codegen::isa::CallConv;
use cranelift_module::Linkage;

impl<'a> CodegenContext<'a> {
    /// Helper to declare and get FuncRef for external math library function
    ///
    /// Always uses TestCase names (even when imports are supported) so that
    /// the fixed-point transformation can detect and replace math functions.
    pub fn get_math_libcall(&mut self, func_name: &str) -> Result<FuncRef, GlslError> {
        // Create signature: f32 -> f32
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::F32));
        sig.returns.push(AbiParam::new(types::F32));

        // Always create TestCase name so fixed-point transformation can detect it
        // This works for both JIT (where imports are supported) and binary compilation
        let sig_ref = self.builder.func.import_signature(sig);
        let ext_name = cranelift_codegen::ir::ExternalName::testcase(func_name.as_bytes());
        let ext_func = cranelift_codegen::ir::ExtFuncData {
            name: ext_name,
            signature: sig_ref,
            colocated: false,
        };
        Ok(self.builder.func.import_function(ext_func))
    }

    /// Helper to declare and get FuncRef for atan2 (2-arg function)
    pub fn get_atan2_libcall(&mut self) -> Result<FuncRef, GlslError> {
        // Create signature: (f32, f32) -> f32
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::F32));
        sig.params.push(AbiParam::new(types::F32));
        sig.returns.push(AbiParam::new(types::F32));

        // Declare function in module if not already declared
        let func_id = self
            .module
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
