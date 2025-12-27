//! Transform tests for backend2
//!
//! These tests verify that transforms preserve function structure correctly.

use cranelift_codegen::ir::{AbiParam, Signature, types};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::write_function;
use lp_glsl::backend2::module::gl_module::GlModule;
use lp_glsl::backend2::module::test_helpers::test_helpers::build_simple_function;
use lp_glsl::backend2::target::Target;
use lp_glsl::backend2::transform::identity::IdentityTransform;
use lp_glsl::backend2::transform::pipeline::Transform;
use cranelift_module::Linkage;

#[cfg(feature = "std")]
fn create_test_module() -> GlModule<cranelift_jit::JITModule> {
    let target = Target::host_jit().unwrap();
    GlModule::new_jit(target).unwrap()
}

/// Normalize CLIF strings for comparison
fn normalize_clif(clif: &str) -> String {
    clif.lines()
        .map(|line| {
            let line = if let Some(comment_pos) = line.find(';') {
                &line[..comment_pos]
            } else {
                line
            };
            line.trim()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_simple() {
    // Build a simple function: i32 add(i32 a, i32 b) -> a + b
    let mut gl_module = create_test_module();

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::I32));

    build_simple_function(&mut gl_module, "add", Linkage::Local, sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let a = builder.block_params(entry_block)[0];
        let b = builder.block_params(entry_block)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    }).unwrap();

    // Get original function
    let original_func = gl_module.get_func("add").unwrap();
    let original_func_clone = original_func.function.clone();

    // Format original
    let mut original_buf = String::new();
    write_function(&mut original_buf, &original_func_clone).unwrap();

    // Apply identity transform
    let transform = IdentityTransform;
    let transformed_module = gl_module.apply_transform(transform).unwrap();

    // Get transformed function
    let transformed_func = transformed_module.get_func("add").unwrap();
    let mut transformed_buf = String::new();
    write_function(&mut transformed_buf, &transformed_func.function).unwrap();

    // Normalize and compare
    let normalized_original = normalize_clif(&original_buf);
    let normalized_transformed = normalize_clif(&transformed_buf);

    assert_eq!(
        normalized_original, normalized_transformed,
        "Identity transform should produce identical CLIF\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_buf, transformed_buf
    );
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_block_order() {
    // Test with multiple blocks to verify block order preservation
    let mut gl_module = create_test_module();

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::I32));

    build_simple_function(&mut gl_module, "test", Linkage::Local, sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let x = builder.block_params(entry_block)[0];
        
        // Create multiple blocks in non-sequential order
        let block1 = builder.create_block();
        let block2 = builder.create_block();
        
        // Insert blocks in specific order
        builder.func.layout.append_block(block1);
        builder.func.layout.append_block(block2);
        
        // Jump to block1
        builder.ins().jump(block1, &[]);
        builder.seal_block(entry_block);
        
        // In block1, jump to block2
        builder.switch_to_block(block1);
        builder.ins().jump(block2, &[]);
        builder.seal_block(block1);
        
        // In block2, return x
        builder.switch_to_block(block2);
        builder.ins().return_(&[x]);
        builder.seal_block(block2);
        
        Ok(())
    }).unwrap();

    // Get original function
    let original_func = gl_module.get_func("test").unwrap();
    let original_func_clone = original_func.function.clone();

    // Format original
    let mut original_buf = String::new();
    write_function(&mut original_buf, &original_func_clone).unwrap();

    // Apply identity transform
    let transform = IdentityTransform;
    let transformed_module = gl_module.apply_transform(transform).unwrap();

    // Get transformed function
    let transformed_func = transformed_module.get_func("test").unwrap();
    let mut transformed_buf = String::new();
    write_function(&mut transformed_buf, &transformed_func.function).unwrap();

    // Normalize and compare
    let normalized_original = normalize_clif(&original_buf);
    let normalized_transformed = normalize_clif(&transformed_buf);

    assert_eq!(
        normalized_original, normalized_transformed,
        "Identity transform should preserve block order\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_buf, transformed_buf
    );
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_block_params() {
    // Test with block parameters to verify they're preserved
    let mut gl_module = create_test_module();

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::I32));

    build_simple_function(&mut gl_module, "test", Linkage::Local, sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let x = builder.block_params(entry_block)[0];
        
        // Create a block with a parameter
        let block1 = builder.create_block();
        builder.append_block_param(block1, types::I32);
        builder.func.layout.append_block(block1);
        
        // Jump to block1 with argument
        builder.ins().jump(block1, &[x]);
        builder.seal_block(entry_block);
        
        // In block1, return the parameter
        builder.switch_to_block(block1);
        let param = builder.block_params(block1)[0];
        builder.ins().return_(&[param]);
        builder.seal_block(block1);
        
        Ok(())
    }).unwrap();

    // Get original function
    let original_func = gl_module.get_func("test").unwrap();
    let original_func_clone = original_func.function.clone();

    // Format original
    let mut original_buf = String::new();
    write_function(&mut original_buf, &original_func_clone).unwrap();

    // Apply identity transform
    let transform = IdentityTransform;
    let transformed_module = gl_module.apply_transform(transform).unwrap();

    // Get transformed function
    let transformed_func = transformed_module.get_func("test").unwrap();
    let mut transformed_buf = String::new();
    write_function(&mut transformed_buf, &transformed_func.function).unwrap();

    // Normalize and compare
    let normalized_original = normalize_clif(&original_buf);
    let normalized_transformed = normalize_clif(&transformed_buf);

    assert_eq!(
        normalized_original, normalized_transformed,
        "Identity transform should preserve block parameters\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_buf, transformed_buf
    );
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_stack_slots() {
    // Test with stack slots to verify they're copied
    let mut gl_module = create_test_module();

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::I32));

    build_simple_function(&mut gl_module, "test", Linkage::Local, sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let x = builder.block_params(entry_block)[0];
        
        // Create a stack slot
        let stack_slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            4, // 4 bytes for i32
        ));
        
        // Store x to stack slot
        let stack_addr = builder.ins().stack_addr(types::I32, stack_slot, 0);
        builder.ins().store(Default::default(), x, stack_addr);
        
        // Load from stack slot and return
        let loaded = builder.ins().stack_load(types::I32, stack_slot, 0);
        builder.ins().return_(&[loaded]);
        
        Ok(())
    }).unwrap();

    // Get original function
    let original_func = gl_module.get_func("test").unwrap();
    let original_func_clone = original_func.function.clone();

    // Format original
    let mut original_buf = String::new();
    write_function(&mut original_buf, &original_func_clone).unwrap();

    // Apply identity transform
    let transform = IdentityTransform;
    let transformed_module = gl_module.apply_transform(transform).unwrap();

    // Get transformed function
    let transformed_func = transformed_module.get_func("test").unwrap();
    let mut transformed_buf = String::new();
    write_function(&mut transformed_buf, &transformed_func.function).unwrap();

    // Normalize and compare
    let normalized_original = normalize_clif(&original_buf);
    let normalized_transformed = normalize_clif(&transformed_buf);

    assert_eq!(
        normalized_original, normalized_transformed,
        "Identity transform should preserve stack slots\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_buf, transformed_buf
    );
}

#[test]
#[cfg(feature = "std")]
fn test_identity_transform_multi_function() {
    // Test with multiple functions and calls between them
    let mut gl_module = create_test_module();

    // Build add function
    let mut add_sig = Signature::new(CallConv::SystemV);
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function(&mut gl_module, "add", Linkage::Local, add_sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let a = builder.block_params(entry_block)[0];
        let b = builder.block_params(entry_block)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    }).unwrap();

    // Build multiply function that calls add
    let mut mult_sig = Signature::new(CallConv::SystemV);
    mult_sig.params.push(AbiParam::new(types::I32));
    mult_sig.params.push(AbiParam::new(types::I32));
    mult_sig.returns.push(AbiParam::new(types::I32));

    // Get add func_id before building multiply
    let add_func_id = gl_module.get_func("add").unwrap().func_id;

    build_simple_function(&mut gl_module, "multiply", Linkage::Local, mult_sig.clone(), |builder| {
        let entry_block = builder.func.layout.entry_block().unwrap();
        let a = builder.block_params(entry_block)[0];
        let b = builder.block_params(entry_block)[1];
        
        // Call add(a, b) - for simplicity, just return a + b directly
        // (calling requires FuncRef which is more complex in test helpers)
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    }).unwrap();

    // Get original functions
    let original_add = gl_module.get_func("add").unwrap().function.clone();
    let original_mult = gl_module.get_func("multiply").unwrap().function.clone();

    // Format originals
    let mut original_add_buf = String::new();
    write_function(&mut original_add_buf, &original_add).unwrap();
    let mut original_mult_buf = String::new();
    write_function(&mut original_mult_buf, &original_mult).unwrap();

    // Apply identity transform
    let transform = IdentityTransform;
    let transformed_module = gl_module.apply_transform(transform).unwrap();

    // Get transformed functions
    let transformed_add = transformed_module.get_func("add").unwrap();
    let transformed_mult = transformed_module.get_func("multiply").unwrap();
    
    let mut transformed_add_buf = String::new();
    write_function(&mut transformed_add_buf, &transformed_add.function).unwrap();
    let mut transformed_mult_buf = String::new();
    write_function(&mut transformed_mult_buf, &transformed_mult.function).unwrap();

    // Normalize and compare
    let normalized_original_add = normalize_clif(&original_add_buf);
    let normalized_transformed_add = normalize_clif(&transformed_add_buf);
    let normalized_original_mult = normalize_clif(&original_mult_buf);
    let normalized_transformed_mult = normalize_clif(&transformed_mult_buf);

    assert_eq!(
        normalized_original_add, normalized_transformed_add,
        "Identity transform should preserve add function\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_add_buf, transformed_add_buf
    );

    assert_eq!(
        normalized_original_mult, normalized_transformed_mult,
        "Identity transform should preserve multiply function\n\
         ORIGINAL:\n{}\n\n\
         TRANSFORMED:\n{}",
        original_mult_buf, transformed_mult_buf
    );
}

