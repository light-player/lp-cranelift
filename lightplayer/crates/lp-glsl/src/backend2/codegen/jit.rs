//! JIT codegen - build executable from GlModule<JITModule>

use crate::backend2::module::gl_module::GlModule;
use crate::exec::jit::GlslJitModule;
use crate::error::{ErrorCode, GlslError};
use cranelift_jit::JITModule;
use hashbrown::HashMap;

/// Build JIT executable from GlModule<JITModule>
/// Called by GlModule<JITModule>::build_executable()
pub fn build_jit_executable(
    mut gl_module: GlModule<JITModule>,
) -> Result<GlslJitModule, GlslError> {
    // 1. Finalize definitions
    gl_module.module.finalize_definitions()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to finalize definitions: {}", e)))?;

    // 2. Extract function pointers
    let mut function_ptrs = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        let ptr = gl_module.module.get_finalized_function(gl_func.func_id);
        function_ptrs.insert(name.clone(), ptr);
    }

    // 3. Build signatures map (minimal for Phase 1)
    let signatures = HashMap::new();
    let mut cranelift_signatures = HashMap::new();
    for (name, gl_func) in &gl_module.fns {
        // For Phase 1, create minimal GLSL signature
        // Full signature support comes later
        cranelift_signatures.insert(name.clone(), gl_func.clif_sig.clone());
    }

    // 4. Get target properties (requires mutable reference for ISA caching)
    let call_conv = gl_module.target.default_call_conv()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to get call conv: {}", e)))?;
    let pointer_type = gl_module.target.pointer_type()
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to get pointer type: {}", e)))?;

    // 5. Create GlslJitModule
    Ok(GlslJitModule {
        jit_module: gl_module.module,
        function_ptrs,
        signatures,
        cranelift_signatures,
        call_conv,
        pointer_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::target::Target;
    use crate::backend2::module::builder::build_simple_function;
    use cranelift_codegen::ir::{types, AbiParam, Signature, InstBuilder};
    use cranelift_codegen::isa::CallConv;
    use cranelift_module::Linkage;

    #[test]
    #[cfg(feature = "std")]
    fn test_build_jit_executable() {
        use crate::exec::executable::GlslExecutable;
        
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();
        
        // Build a simple function that returns 42
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));
        
        build_simple_function(&mut gl_module, "main", Linkage::Export, sig, |builder| {
            let val = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[val]);
            Ok(())
        }).unwrap();
        
        // Build executable
        let mut executable = build_jit_executable(gl_module).unwrap();
        assert!(executable.function_ptrs.contains_key("main"));
        
        // Actually call the function and verify it returns 42
        let result = executable.call_i32("main", &[]).unwrap();
        assert_eq!(result, 42);
    }
}
