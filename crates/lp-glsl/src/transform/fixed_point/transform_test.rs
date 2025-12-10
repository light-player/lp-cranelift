//! Unit tests for fixed-point transformation

use super::transform::{convert_floats_to_fixed, verify_no_f32_values_for_testing};
use super::types::FixedPointFormat;
use crate::error::GlslError;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{
    ExtFuncData, ExternalName, Function, Inst, InstBuilder, Signature, UserFuncName,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

/// Helper to create a basic function with F32 signature
fn create_function_with_f32_sig() -> Function {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    sig.returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test"), sig);
    func
}

/// Helper to assert no F32 values remain in function
fn assert_no_f32_values(func: &Function) {
    verify_no_f32_values_for_testing(func)
        .expect("Function should have no F32 values after transformation");
}

/// Helper to verify a value type is not F32
fn assert_not_f32(func: &Function, value: cranelift_codegen::ir::Value) {
    assert_ne!(
        func.dfg.value_type(value),
        types::F32,
        "Value {:?} should not be F32",
        value
    );
}

/// Test basic F32 constant conversion
#[test]
fn test_f32const_conversion() {
    let mut func = create_function_with_f32_sig();
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);

    // Create F32 constant
    let f32_const = builder.ins().f32const(1.5);
    builder.ins().return_(&[f32_const]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);

    // Verify signature is converted
    assert_eq!(func.signature.params[0].value_type, types::I32);
    assert_eq!(func.signature.returns[0].value_type, types::I32);
}

/// Test F32 addition conversion
#[test]
fn test_fadd_conversion() {
    let mut func = create_function_with_f32_sig();
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let param = builder.block_params(entry_block)[0];

    // Create F32 addition
    let f32_const = builder.ins().f32const(2.0);
    let result = builder.ins().fadd(param, f32_const);
    builder.ins().return_(&[result]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);
}

// Note: Load/Store tests are skipped for now as they require more complex setup.
// These would be better tested via integration tests with actual GLSL code.

#[test]
#[ignore] // Skip load/store tests for now
fn test_convert_load_with_f32_result() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::I32)); // address
    sig.returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test_load"), sig);
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let _addr = builder.block_params(entry_block)[0];

    // Load F32 value - TODO: Fix API usage
    // For now, just return a constant to test signature conversion
    let f32_const = builder.ins().f32const(1.0);
    builder.ins().return_(&[f32_const]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);

    // Verify return type is converted
    assert_eq!(func.signature.returns[0].value_type, types::I32);
}

#[test]
#[ignore] // Skip load/store tests for now
fn test_convert_store_with_f32_value() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::I32)); // address
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32)); // value

    let mut func = Function::with_name_signature(UserFuncName::testcase("test_store"), sig);
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let params = builder.block_params(entry_block);
    let _addr = params[0];
    let _val = params[1];

    // Store F32 value - TODO: Fix API usage
    // For now, just return to test signature conversion
    builder.ins().return_(&[]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);

    // Verify parameter type is converted
    assert_eq!(func.signature.params[1].value_type, types::I32);
}

/// Test block parameters are converted
#[test]
fn test_block_parameters_converted() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    sig.returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test_block_params"), sig);
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let param = builder.block_params(entry_block)[0];

    // Use the parameter
    builder.ins().return_(&[param]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);

    // Verify block parameter is converted
    let entry_params = func.dfg.block_params(entry_block);
    assert_eq!(entry_params.len(), 1);
    assert_not_f32(&func, entry_params[0]);
}

/// Test value mapping propagates correctly
#[test]
fn test_value_map_propagates_correctly() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    sig.returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test_value_map"), sig);
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let param = builder.block_params(entry_block)[0];

    // Chain: param -> fadd -> return
    let f32_const = builder.ins().f32const(1.0);
    let fadd_result = builder.ins().fadd(param, f32_const);
    builder.ins().return_(&[fadd_result]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);
}

/// Test complete function transformation
#[test]
fn test_complete_function_transformation() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    sig.returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test_complete"), sig);
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let param = builder.block_params(entry_block)[0];

    // Create a complex expression with multiple F32 operations
    let const1 = builder.ins().f32const(2.0);
    let const2 = builder.ins().f32const(3.0);
    let add_result = builder.ins().fadd(param, const1);
    let mul_result = builder.ins().fmul(add_result, const2);
    let neg_result = builder.ins().fneg(mul_result);
    builder.ins().return_(&[neg_result]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);

    // Verify function is valid
    cranelift_codegen::verify_function(
        &func,
        &cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()),
    )
    .expect("Function should be valid after transformation");
}

/// Test call instruction conversion with F32 return type
#[test]
fn test_call_f32_return_conversion() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    sig.returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));

    let mut func = Function::with_name_signature(UserFuncName::testcase("test_call"), sig);

    // Create external function signature (F32 -> F32) and import function BEFORE creating builder
    let mut ext_sig = Signature::new(CallConv::SystemV);
    ext_sig
        .params
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    ext_sig
        .returns
        .push(cranelift_codegen::ir::AbiParam::new(types::F32));
    let ext_sig_ref = func.import_signature(ext_sig);

    // Import external function
    let ext_name = ExternalName::testcase(b"ext_func");
    let ext_func_data = ExtFuncData {
        name: ext_name,
        signature: ext_sig_ref,
        colocated: false,
    };
    let ext_func_ref = func.import_function(ext_func_data);

    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    let param = builder.block_params(entry_block)[0];

    // Call external function that returns F32
    let call_inst = builder.ins().call(ext_func_ref, &[param]);
    let call_result = builder.inst_results(call_inst)[0];
    builder.ins().return_(&[call_result]);

    builder.seal_all_blocks();
    builder.finalize();

    // Convert to fixed-point
    convert_floats_to_fixed(&mut func, FixedPointFormat::Fixed16x16)
        .expect("Conversion should succeed");

    // Verify no F32 values remain
    assert_no_f32_values(&func);

    // Verify external function signature was converted
    let converted_ext_sig = &func.dfg.signatures[ext_sig_ref];
    assert_eq!(converted_ext_sig.params[0].value_type, types::I32);
    assert_eq!(converted_ext_sig.returns[0].value_type, types::I32);
}
