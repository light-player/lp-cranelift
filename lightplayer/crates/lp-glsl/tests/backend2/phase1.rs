//! Phase 1: Proof of Concept - Direct Module Building Tests
//!
//! These tests validate the core concept: build functions directly in the final Module
//! without a linking step, and prove both JIT and emulator backends work.

use lp_glsl::backend2::target::TargetSpec;
use lp_glsl::backend2::module::GlJitModule;
#[cfg(feature = "emulator")]
use lp_glsl::backend2::module::GlObjectModule;
use lp_glsl::backend2::module::builder::build_simple_function_jit;
#[cfg(feature = "emulator")]
use lp_glsl::backend2::module::builder::build_simple_function_object;
use lp_glsl::backend2::codegen::{build_jit_executable, build_emu_executable};
#[cfg(feature = "emulator")]
use lp_glsl::backend2::codegen::emu::EmulatorOptions;
use lp_glsl::{GlslExecutable, GlslValue};
use lp_glsl::error::GlslError;
use cranelift_codegen::ir::{types, AbiParam, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_module::Linkage;

/// Build the test functions (add and main) into a GlJitModule
/// This simulates what the frontend will do - build functions directly in the Module
fn build_test_functions_jit(gl_module: &mut GlJitModule) -> Result<(), GlslError> {
    // Build helper: add(a: i32, b: i32) -> i32
    let mut add_sig = Signature::new(CallConv::SystemV);
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_jit(gl_module, "add", Linkage::Local, add_sig, |builder, _module| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    })?;

    // Build main: main() -> add(10, 20)
    let mut main_sig = Signature::new(CallConv::SystemV);
    main_sig.returns.push(AbiParam::new(types::I32));

    // Extract func_id before closure to avoid borrowing conflicts
    let add_func_id = gl_module.get_func("add")
        .ok_or_else(|| GlslError::new(lp_glsl::error::ErrorCode::E0400, "Function 'add' not found"))?
        .func_id;

    build_simple_function_jit(gl_module, "main", Linkage::Export, main_sig, move |builder, module| {
        let ten = builder.ins().iconst(types::I32, 10);
        let twenty = builder.ins().iconst(types::I32, 20);

        // Create FuncRef using the captured func_id
        let add_ref = module.declare_func_in_func(add_func_id, builder.func);

        // Call add(10, 20)
        let call_result = builder.ins().call(add_ref, &[ten, twenty]);
        let result = builder.inst_results(call_result)[0];

        builder.ins().return_(&[result]);
        Ok(())
    })?;

    Ok(())
}

/// Build the test functions for GlObjectModule
#[cfg(feature = "emulator")]
fn build_test_functions_object(gl_module: &mut GlObjectModule) -> Result<(), GlslError> {
    // Same as JIT version but for ObjectModule
    let mut add_sig = Signature::new(CallConv::SystemV);
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_object(gl_module, "add", Linkage::Local, add_sig, |builder, _module| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    })?;

    let mut main_sig = Signature::new(CallConv::SystemV);
    main_sig.returns.push(AbiParam::new(types::I32));

    // Extract func_id before closure to avoid borrowing conflicts
    let add_func_id = gl_module.get_func("add")
        .ok_or_else(|| GlslError::new(lp_glsl::error::ErrorCode::E0400, "Function 'add' not found"))?
        .func_id;

    // Build main: main() -> add(10, 20)
    build_simple_function_object(gl_module, "main", Linkage::Export, main_sig, move |builder, module| {
        let ten = builder.ins().iconst(types::I32, 10);
        let twenty = builder.ins().iconst(types::I32, 20);

        // Create FuncRef using the captured func_id
        let add_ref = module.declare_func_in_func(add_func_id, builder.func);

        // Call add(10, 20)
        let call_result = builder.ins().call(add_ref, &[ten, twenty]);
        let result = builder.inst_results(call_result)[0];

        builder.ins().return_(&[result]);
        Ok(())
    })?;

    Ok(())
}

/// Test 1: JIT - Simple Function Call
#[test]
#[cfg(feature = "std")]
fn test_jit_function_call() {
    // Create JIT target spec
    let target = TargetSpec::host_jit().expect("Failed to create JIT target spec");

    // Create GlModule
    let mut gl_module = GlJitModule::new(target).expect("Failed to create GlJitModule");

    // Build functions (same logic works for both JIT and emulator!)
    build_test_functions_jit(&mut gl_module).expect("Failed to build test functions");

    // Build executable and test
    let mut executable = build_jit_executable(gl_module).expect("Failed to build JIT executable");
    let result = executable.call_i32("main", &[]).expect("Failed to call main");
    assert_eq!(result, 30);
}

/// Test 2: Emulator - Simple Function Call
#[test]
#[cfg(feature = "emulator")]
fn test_emu_function_call() {
    // Create emulator target spec
    let target = TargetSpec::riscv32_emulator().expect("Failed to create emulator target spec");

    // Create GlModule
    let mut gl_module = GlObjectModule::new(target).expect("Failed to create GlObjectModule");

    // Build functions (same logic as JIT - no changes needed!)
    build_test_functions_object(&mut gl_module).expect("Failed to build test functions");

    // Build executable and test
    let options = EmulatorOptions {
        max_memory: 1024 * 1024,
        stack_size: 64 * 1024,
        max_instructions: 10000,
    };
    let mut executable = build_emu_executable(gl_module, &options).expect("Failed to build emulator executable");
    let result = executable.call_i32("main", &[]).expect("Failed to call main");
    assert_eq!(result, 30);
}

/// Build more complex test with multiple function calls
fn build_multiple_call_test_jit(gl_module: &mut GlJitModule) -> Result<(), GlslError> {
    // Build multiply(a: i32, b: i32) -> i32
    let mut mult_sig = Signature::new(CallConv::SystemV);
    mult_sig.params.push(AbiParam::new(types::I32));
    mult_sig.params.push(AbiParam::new(types::I32));
    mult_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_jit(gl_module, "multiply", Linkage::Local, mult_sig, |builder, _module| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let product = builder.ins().imul(a, b);
        builder.ins().return_(&[product]);
        Ok(())
    })?;

    // We still need the add function
    let mut add_sig = Signature::new(CallConv::SystemV);
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_jit(gl_module, "add", Linkage::Local, add_sig, |builder, _module| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    })?;

    // Extract func_ids before closure to avoid borrowing conflicts
    let mult_func_id = gl_module.get_func("multiply")
        .ok_or_else(|| GlslError::new(lp_glsl::error::ErrorCode::E0400, "Function 'multiply' not found"))?
        .func_id;
    let add_func_id = gl_module.get_func("add")
        .ok_or_else(|| GlslError::new(lp_glsl::error::ErrorCode::E0400, "Function 'add' not found"))?
        .func_id;

    // Build main: main() -> add(multiply(2, 3), multiply(4, 5))
    // Should return 2*3 + 4*5 = 6 + 20 = 26
    let mut main_sig = Signature::new(CallConv::SystemV);
    main_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_jit(gl_module, "main", Linkage::Export, main_sig, move |builder, module| {
        // Call multiply(2, 3)
        let two = builder.ins().iconst(types::I32, 2);
        let three = builder.ins().iconst(types::I32, 3);
        let mult_ref = module.declare_func_in_func(mult_func_id, builder.func);
        let call1 = builder.ins().call(mult_ref, &[two, three]);
        let result1 = builder.inst_results(call1)[0];

        // Call multiply(4, 5)
        let four = builder.ins().iconst(types::I32, 4);
        let five = builder.ins().iconst(types::I32, 5);
        let call2 = builder.ins().call(mult_ref, &[four, five]);
        let result2 = builder.inst_results(call2)[0];

        // Call add(result1, result2)
        let add_ref = module.declare_func_in_func(add_func_id, builder.func);
        let call3 = builder.ins().call(add_ref, &[result1, result2]);
        let final_result = builder.inst_results(call3)[0];

        builder.ins().return_(&[final_result]);
        Ok(())
    })?;

    Ok(())
}

/// Build multiple call test for ObjectModule
#[cfg(feature = "emulator")]
fn build_multiple_call_test_object(gl_module: &mut GlObjectModule) -> Result<(), GlslError> {
    // Same as JIT version
    let mut mult_sig = Signature::new(CallConv::SystemV);
    mult_sig.params.push(AbiParam::new(types::I32));
    mult_sig.params.push(AbiParam::new(types::I32));
    mult_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_object(gl_module, "multiply", Linkage::Local, mult_sig, |builder, _module| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let product = builder.ins().imul(a, b);
        builder.ins().return_(&[product]);
        Ok(())
    })?;

    // We still need the add function
    let mut add_sig = Signature::new(CallConv::SystemV);
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.params.push(AbiParam::new(types::I32));
    add_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_object(gl_module, "add", Linkage::Local, add_sig, |builder, _module| {
        let entry = builder.current_block().unwrap();
        let a = builder.block_params(entry)[0];
        let b = builder.block_params(entry)[1];
        let sum = builder.ins().iadd(a, b);
        builder.ins().return_(&[sum]);
        Ok(())
    })?;

    // Extract func_ids before closure to avoid borrowing conflicts
    let mult_func_id = gl_module.get_func("multiply")
        .ok_or_else(|| GlslError::new(lp_glsl::error::ErrorCode::E0400, "Function 'multiply' not found"))?
        .func_id;
    let add_func_id = gl_module.get_func("add")
        .ok_or_else(|| GlslError::new(lp_glsl::error::ErrorCode::E0400, "Function 'add' not found"))?
        .func_id;

    let mut main_sig = Signature::new(CallConv::SystemV);
    main_sig.returns.push(AbiParam::new(types::I32));

    build_simple_function_object(gl_module, "main", Linkage::Export, main_sig, move |builder, module| {
        let two = builder.ins().iconst(types::I32, 2);
        let three = builder.ins().iconst(types::I32, 3);
        let mult_ref = module.declare_func_in_func(mult_func_id, builder.func);
        let call1 = builder.ins().call(mult_ref, &[two, three]);
        let result1 = builder.inst_results(call1)[0];

        let four = builder.ins().iconst(types::I32, 4);
        let five = builder.ins().iconst(types::I32, 5);
        let call2 = builder.ins().call(mult_ref, &[four, five]);
        let result2 = builder.inst_results(call2)[0];

        let add_ref = module.declare_func_in_func(add_func_id, builder.func);
        let call3 = builder.ins().call(add_ref, &[result1, result2]);
        let final_result = builder.inst_results(call3)[0];

        builder.ins().return_(&[final_result]);
        Ok(())
    })?;

    Ok(())
}

#[test]
#[cfg(feature = "std")]
fn test_jit_multiple_calls() {
    let target = TargetSpec::host_jit().expect("Failed to create JIT target spec");
    let mut gl_module = GlJitModule::new(target).expect("Failed to create GlJitModule");
    build_multiple_call_test_jit(&mut gl_module).expect("Failed to build multiple call test");

    let mut executable = build_jit_executable(gl_module).expect("Failed to build JIT executable");
    let result = executable.call_i32("main", &[]).expect("Failed to call main");
    assert_eq!(result, 26);
}

#[test]
#[cfg(feature = "emulator")]
fn test_emu_multiple_calls() {
    let target = TargetSpec::riscv32_emulator().expect("Failed to create emulator target spec");
    let mut gl_module = GlObjectModule::new(target).expect("Failed to create GlObjectModule");
    build_multiple_call_test_object(&mut gl_module).expect("Failed to build multiple call test");

    let options = EmulatorOptions {
        max_memory: 1024 * 1024,
        stack_size: 64 * 1024,
        max_instructions: 10000,
    };
    let mut executable = build_emu_executable(gl_module, &options).expect("Failed to build emulator executable");
    let result = executable.call_i32("main", &[]).expect("Failed to call main");
    assert_eq!(result, 26);
}
