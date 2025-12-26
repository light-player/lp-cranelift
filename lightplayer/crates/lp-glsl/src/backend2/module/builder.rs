//! Programmatic function building utilities

use crate::backend2::module::gl_module::GlModule;
use crate::backend2::module::gl_func::GlFunc;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{Signature, Value, InstBuilder};
use cranelift_frontend::FunctionBuilder;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_module::{Module, FuncId, Linkage};
use alloc::vec::Vec;
use alloc::string::String;

/// Build a simple function programmatically
/// 
/// **Note**: Function must be declared before it can be called by other functions.
/// Use `declare_function` first if you need to call this function from another.
pub fn build_simple_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
    body: impl FnOnce(&mut FunctionBuilder) -> Result<(), GlslError>,
) -> Result<FuncId, GlslError> {
    // 1. Declare function in module
    let func_id = gl_module.module_mut().declare_function(name, linkage, &sig)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to declare function '{}': {}", name, e)))?;

    // 2. Create context and builder
    let mut ctx = gl_module.module_mut().make_context();
    
    // 3. Set signature and name (before creating builder)
    ctx.func.signature = sig.clone();
    ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());
    
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    // 4. Build entry block
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // 5. Call user-provided body builder
    body(&mut builder)?;

    // 6. Finalize and define
    builder.finalize();
    gl_module.module_mut().define_function(func_id, &mut ctx)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to define function '{}': {}", name, e)))?;
    gl_module.module_mut().clear_context(&mut ctx);

    // 7. Store metadata
    gl_module.fns.insert(String::from(name), GlFunc {
        name: String::from(name),
        clif_sig: sig,
        func_id,
    });

    Ok(func_id)
}

/// Build a function that calls another function
/// 
/// **Note**: The callee function must be declared before calling this function.
/// The callee should be built using `build_simple_function` or `declare_function` first.
pub fn build_call_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
    callee_name: &str,
    args_builder: impl FnOnce(&mut FunctionBuilder) -> Result<Vec<Value>, GlslError>,
) -> Result<FuncId, GlslError> {
    // Get callee FuncId before entering closure (to avoid borrowing issues)
    let callee_func_id = gl_module.get_func(callee_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("Function '{}' not found (must be declared first)", callee_name)))?
        .func_id;
    
    // Declare function first
    let func_id = gl_module.module_mut().declare_function(name, linkage, &sig)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to declare function '{}': {}", name, e)))?;

    // Create context and set up function
    let mut ctx = gl_module.module_mut().make_context();
    ctx.func.signature = sig.clone();
    ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());
    
    // Create FuncRef BEFORE creating builder (to avoid borrowing conflicts)
    let callee_ref = gl_module.module_mut().declare_func_in_func(callee_func_id, &mut ctx.func);
    
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    // Build entry block
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // Build arguments using user-provided builder
    let args = args_builder(&mut builder)?;

    // Call the function
    let call_inst = builder.ins().call(callee_ref, &args);
    // Get results before using builder mutably again
    let result_values: Vec<Value> = builder.inst_results(call_inst).to_vec();
    
    if result_values.is_empty() {
        builder.ins().return_(&[]);
    } else {
        builder.ins().return_(&[result_values[0]]);
    }

    // Finalize and define
    builder.finalize();
    gl_module.module_mut().define_function(func_id, &mut ctx)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to define function '{}': {}", name, e)))?;
    gl_module.module_mut().clear_context(&mut ctx);

    // Store metadata
    gl_module.fns.insert(String::from(name), GlFunc {
        name: String::from(name),
        clif_sig: sig,
        func_id,
    });

    Ok(func_id)
}

/// Declare a function without defining it (useful for forward declarations)
/// Returns the FuncId for later use in function calls
pub fn declare_function<M: Module>(
    gl_module: &mut GlModule<M>,
    name: &str,
    linkage: Linkage,
    sig: Signature,
) -> Result<FuncId, GlslError> {
    let func_id = gl_module.module_mut().declare_function(name, linkage, &sig)
        .map_err(|e| GlslError::new(ErrorCode::E0400, format!("Failed to declare function '{}': {}", name, e)))?;
    
    gl_module.fns.insert(String::from(name), GlFunc {
        name: String::from(name),
        clif_sig: sig,
        func_id,
    });
    
    Ok(func_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend2::target::Target;
    use cranelift_codegen::ir::{types, AbiParam, Signature, InstBuilder};
    use cranelift_codegen::isa::CallConv;
    use cranelift_module::Linkage;

    #[test]
    #[cfg(feature = "std")]
    fn test_build_simple_function() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();
        
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));
        
        let result = build_simple_function(&mut gl_module, "test", Linkage::Local, sig, |builder| {
            let entry = builder.current_block().unwrap();
            let val = builder.ins().iconst(types::I32, 42);
            builder.ins().return_(&[val]);
            Ok(())
        });
        
        assert!(result.is_ok());
        assert!(gl_module.get_func("test").is_some());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_build_function_with_params() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();
        
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I32));
        sig.params.push(AbiParam::new(types::I32));
        sig.returns.push(AbiParam::new(types::I32));
        
        let result = build_simple_function(&mut gl_module, "add", Linkage::Local, sig, |builder| {
            let entry = builder.current_block().unwrap();
            let a = builder.block_params(entry)[0];
            let b = builder.block_params(entry)[1];
            let sum = builder.ins().iadd(a, b);
            builder.ins().return_(&[sum]);
            Ok(())
        });
        
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_build_call_function() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();
        
        // First, build the callee
        let mut add_sig = Signature::new(CallConv::SystemV);
        add_sig.params.push(AbiParam::new(types::I32));
        add_sig.params.push(AbiParam::new(types::I32));
        add_sig.returns.push(AbiParam::new(types::I32));
        
        build_simple_function(&mut gl_module, "add", Linkage::Local, add_sig, |builder| {
            let entry = builder.current_block().unwrap();
            let a = builder.block_params(entry)[0];
            let b = builder.block_params(entry)[1];
            let sum = builder.ins().iadd(a, b);
            builder.ins().return_(&[sum]);
            Ok(())
        }).unwrap();
        
        // Then, build the caller
        let mut main_sig = Signature::new(CallConv::SystemV);
        main_sig.returns.push(AbiParam::new(types::I32));
        
        let result = build_call_function(&mut gl_module, "main", Linkage::Export, main_sig, "add", |builder| {
            let ten = builder.ins().iconst(types::I32, 10);
            let twenty = builder.ins().iconst(types::I32, 20);
            Ok(vec![ten, twenty])
        });
        
        assert!(result.is_ok());
        assert!(gl_module.get_func("main").is_some());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_declare_function() {
        let target = Target::host_jit().unwrap();
        let mut gl_module = GlModule::new_jit(target).unwrap();
        
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I32));
        
        let result = declare_function(&mut gl_module, "forward", Linkage::Local, sig);
        assert!(result.is_ok());
        assert!(gl_module.get_func("forward").is_some());
    }
}
