//! Unit tests for the fixed-point rewrite system.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::rewrite;
use crate::transform::fixed_point::types::FixedPointFormat;

use cranelift_codegen::ir::{
    AbiParam, BlockArg, Function, InstBuilder, Signature, UserFuncName,
    condcodes::{FloatCC, IntCC},
    types,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

// Use the public API from transform module
use crate::transform::fixed_point::convert_floats_to_fixed;

/// Helper to create a simple function with F32 operations
fn create_simple_f32_function() -> Function {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    {
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        let params = {
            let params_slice = builder.block_params(block);
            params_slice.to_vec()
        };
        let param = params[0];
        let const_val = builder.ins().f32const(1.0);
        let result = builder.ins().fadd(param, const_val);
        builder.ins().return_(&[result]);
        builder.seal_all_blocks();
        builder.finalize();
    } // Drop builder and context here

    func
}

#[test]
fn test_simple_function_rewrite() {
    let mut func = create_simple_f32_function();
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");

    // Verify function is valid Cranelift IR
    cranelift_codegen::verify_function(
        &func,
        &cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()),
    )
    .expect("Function should be valid");
}

// Note: Verification tests are in transform.rs since verify_no_f32_values is private
// These tests use the public API which includes verification

#[test]
fn test_arithmetic_operations() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let params = {
        let params_slice = builder.block_params(block);
        params_slice.to_vec()
    };
    let add_result = builder.ins().fadd(params[0], params[1]);
    let sub_result = builder.ins().fsub(params[0], params[1]);
    let mul_result = builder.ins().fmul(add_result, sub_result);
    let div_result = builder.ins().fdiv(mul_result, params[0]);
    builder.ins().return_(&[div_result]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_comparison_operations() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::I32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let params = {
        let params_slice = builder.block_params(block);
        params_slice.to_vec()
    };
    let cmp = builder
        .ins()
        .fcmp(FloatCC::GreaterThan, params[0], params[1]);
    builder.ins().return_(&[cmp]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_fmax_fmin_operations() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let params = {
        let params_slice = builder.block_params(block);
        params_slice.to_vec()
    };
    let max_result = builder.ins().fmax(params[0], params[1]);
    let min_result = builder.ins().fmin(max_result, params[0]);
    builder.ins().return_(&[min_result]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_ceil_floor_operations() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let params = {
        let params_slice = builder.block_params(block);
        params_slice.to_vec()
    };
    let param = params[0];
    let ceil_result = builder.ins().ceil(param);
    let floor_result = builder.ins().floor(ceil_result);
    builder.ins().return_(&[floor_result]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_type_conversions() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let params = {
        let params_slice = builder.block_params(block);
        params_slice.to_vec()
    };
    let param = params[0];
    let converted = builder.ins().fcvt_from_sint(types::F32, param);
    builder.ins().return_(&[converted]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_control_flow() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block0 = builder.create_block();
    let block1 = builder.create_block();
    let block2 = builder.create_block();

    builder.append_block_params_for_function_params(block0);
    builder.switch_to_block(block0);
    let params = {
        let params_slice = builder.block_params(block0);
        params_slice.to_vec()
    };
    let param = params[0];
    let zero = builder.ins().f32const(0.0);
    let cmp = builder.ins().fcmp(FloatCC::GreaterThan, param, zero);
    builder.ins().brif(cmp, block1, &[], block2, &[]);

    builder.switch_to_block(block1);
    let one = builder.ins().f32const(1.0);
    builder.append_block_param(block2, types::F32);
    let one_arg: BlockArg = one.into();
    builder.ins().jump(block2, &[one_arg]);

    builder.switch_to_block(block2);
    builder.seal_block(block2);
    let phi = builder.block_params(block2)[0];
    builder.ins().return_(&[phi]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_select_instruction() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.params.push(AbiParam::new(types::F32));
    sig.params.push(AbiParam::new(types::I32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let params = {
        let params_slice = builder.block_params(block);
        params_slice.to_vec()
    };
    let zero_const = builder.ins().iconst(types::I32, 0);
    let cond = builder.ins().icmp(IntCC::NotEqual, params[2], zero_const);
    let result = builder.ins().select(cond, params[0], params[1]);
    builder.ins().return_(&[result]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_memory_operations() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I32)); // address
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let addr = builder.block_params(block)[0];
    let loaded = builder.ins().load(
        types::F32,
        cranelift_codegen::ir::MemFlags::trusted(),
        addr,
        0,
    );
    builder.ins().return_(&[loaded]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");
}

#[test]
fn test_complex_function_rewrite() {
    // Test complex function with multiple operations
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::F32));
    sig.params.push(AbiParam::new(types::F32));
    sig.returns.push(AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

    let block0 = builder.create_block();
    let block1 = builder.create_block();
    let block2 = builder.create_block();

    builder.append_block_params_for_function_params(block0);
    builder.switch_to_block(block0);
    let params = {
        let params_slice = builder.block_params(block0);
        params_slice.to_vec()
    };

    // Arithmetic
    let add = builder.ins().fadd(params[0], params[1]);
    let mul = builder.ins().fmul(add, params[0]);

    // Comparison
    let zero = builder.ins().f32const(0.0);
    let cmp = builder.ins().fcmp(FloatCC::GreaterThan, mul, zero);
    builder.ins().brif(cmp, block1, &[], block2, &[]);

    builder.switch_to_block(block1);
    let ceil_val = builder.ins().ceil(mul);
    builder.append_block_param(block2, types::F32);
    let ceil_arg: BlockArg = ceil_val.into();
    builder.ins().jump(block2, &[ceil_arg]);

    builder.switch_to_block(block2);
    builder.seal_block(block2);
    let phi = builder.block_params(block2)[0];
    let floor_val = builder.ins().floor(phi);
    builder.ins().return_(&[floor_val]);
    builder.seal_all_blocks();
    builder.finalize();

    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Rewrite should succeed");

    // Verify function is valid
    cranelift_codegen::verify_function(
        &func,
        &cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()),
    )
    .expect("Function should be valid");
}

#[test]
fn test_all_comparison_conditions() {
    // Test that all FloatCC conditions are converted correctly
    let conditions = vec![
        FloatCC::Equal,
        FloatCC::NotEqual,
        FloatCC::LessThan,
        FloatCC::LessThanOrEqual,
        FloatCC::GreaterThan,
        FloatCC::GreaterThanOrEqual,
    ];

    for cond in conditions {
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::F32));
        sig.params.push(AbiParam::new(types::F32));
        sig.returns.push(AbiParam::new(types::I32));

        let mut func =
            Function::with_name_signature(UserFuncName::testcase(&format!("test_{:?}", cond)), sig);

        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        let params = {
            let params_slice = builder.block_params(block);
            params_slice.to_vec()
        };
        let cmp = builder.ins().fcmp(cond, params[0], params[1]);
        builder.ins().return_(&[cmp]);
        builder.finalize();

        convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
            .expect(&format!("Rewrite should succeed for condition {:?}", cond));
    }
}

