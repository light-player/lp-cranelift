//! Comparison operation conversion functions.

use crate::error::GlslError;
use crate::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::{
    Function, Inst, InstBuilder, InstructionData, Value,
    condcodes::{FloatCC, IntCC},
};
use cranelift_frontend::FunctionBuilder;

use super::{extract_binary_operands, get_first_result, map_operand, unexpected_format_error};

/// Convert Fcmp to icmp.
///
/// Note: Fixed-point arithmetic does not have NaN or Infinity values.
/// FloatCC conditions that check for NaN/Inf (Ordered/Unordered) are approximated
/// using integer comparisons, which may not match floating-point behavior exactly
/// for edge cases involving NaN or Infinity.
pub(crate) fn convert_fcmp(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    _format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::FloatCompare { cond, args, .. } = inst_data {
        // Map operands
        let arg1 = map_operand(value_map, args[0]);
        let arg2 = map_operand(value_map, args[1]);

        // Convert float condition to integer condition
        let int_cond = match cond {
            FloatCC::Equal => IntCC::Equal,
            FloatCC::NotEqual => IntCC::NotEqual,
            FloatCC::LessThan => IntCC::SignedLessThan,
            FloatCC::LessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::GreaterThan => IntCC::SignedGreaterThan,
            FloatCC::GreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
            // Note: Ordered/Unordered conditions are approximated since fixed-point
            // doesn't have NaN. These approximations may not match float behavior exactly.
            FloatCC::Ordered => IntCC::Equal,      // Approximate: always true for fixed-point
            FloatCC::Unordered => IntCC::NotEqual, // Approximate: always false for fixed-point
            FloatCC::OrderedNotEqual => IntCC::NotEqual,
            FloatCC::UnorderedOrEqual => IntCC::Equal,
            FloatCC::UnorderedOrLessThan => IntCC::SignedLessThan,
            FloatCC::UnorderedOrLessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::UnorderedOrGreaterThan => IntCC::SignedGreaterThan,
            FloatCC::UnorderedOrGreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
        };

        // Emit icmp (returns i8)
        let cmp_result = builder.ins().icmp(int_cond, arg1, arg2);

        // Always convert i8 to i32 for fcmp results
        // fcmp returns i8, but since we're converting from F32 operations,
        // the function signature will expect I32 (F32 -> I32 conversion)
        // So we always convert i8 to i32 to match
        let new_result = builder
            .ins()
            .sextend(cranelift_codegen::ir::types::I32, cmp_result);

        // Map result
        let old_result = get_first_result(old_func, old_inst);
        value_map.insert(old_result, new_result);
    } else {
        return Err(unexpected_format_error(old_func, old_inst, "FloatCompare"));
    }

    Ok(())
}

/// Convert Fmax to select with comparison.
pub(crate) fn convert_fmax(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    _format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Compare and select maximum
    let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, arg1, arg2);
    let new_result = builder.ins().select(cmp, arg1, arg2);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fmin to select with comparison.
pub(crate) fn convert_fmin(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    _format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Compare and select minimum
    let cmp = builder.ins().icmp(IntCC::SignedLessThan, arg1, arg2);
    let new_result = builder.ins().select(cmp, arg1, arg2);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}
