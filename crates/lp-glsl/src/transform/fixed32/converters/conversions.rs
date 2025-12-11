//! Type conversion instruction conversion functions.

use crate::error::GlslError;
use crate::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, Value, types};
use cranelift_frontend::FunctionBuilder;

use crate::ir_utils::instruction::{extract_unary_operand, get_first_result};
use crate::ir_utils::value_map::map_operand;

/// Convert FcvtFromSint instruction.
pub(crate) fn convert_fcvt_from_sint(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    // Check if result is F32
    let old_result = get_first_result(old_func, old_inst);
    if old_func.dfg.value_type(old_result) != types::F32 {
        // Not an F32 conversion, skip
        return Ok(());
    }

    let arg = extract_unary_operand(old_func, old_inst)?;

    // Map argument
    let mapped_arg = map_operand(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Convert integer to fixed-point: int << shift_amount
    // Need to sign-extend if arg is smaller than target_type
    let arg_type = old_func.dfg.value_type(arg);
    let shift_const = builder.ins().iconst(target_type, shift_amount as i64);

    let shifted = if arg_type.bits() < target_type.bits() {
        // Sign-extend first, then shift
        let extended = builder.ins().sextend(target_type, mapped_arg);
        builder.ins().ishl(extended, shift_const)
    } else {
        // Direct shift
        builder.ins().ishl(mapped_arg, shift_const)
    };

    value_map.insert(old_result, shifted);

    Ok(())
}

/// Convert FcvtFromUint instruction.
pub(crate) fn convert_fcvt_from_uint(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    // Check if result is F32
    let old_result = get_first_result(old_func, old_inst);
    if old_func.dfg.value_type(old_result) != types::F32 {
        // Not an F32 conversion, skip
        return Ok(());
    }

    let arg = extract_unary_operand(old_func, old_inst)?;

    // Map argument
    let mapped_arg = map_operand(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Convert unsigned integer to fixed-point: uint << shift_amount
    // Need to zero-extend if arg is smaller than target_type
    let arg_type = old_func.dfg.value_type(arg);
    let shift_const = builder.ins().iconst(target_type, shift_amount as i64);

    let shifted = if arg_type.bits() < target_type.bits() {
        // Zero-extend first, then shift
        let extended = builder.ins().uextend(target_type, mapped_arg);
        builder.ins().ishl(extended, shift_const)
    } else {
        // Direct shift
        builder.ins().ishl(mapped_arg, shift_const)
    };

    value_map.insert(old_result, shifted);

    Ok(())
}

