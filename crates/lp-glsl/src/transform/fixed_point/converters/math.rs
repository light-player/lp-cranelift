//! Math function conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};
use cranelift_frontend::FunctionBuilder;

use super::super::rewrite::map_value;

/// Convert Ceil instruction.
pub(crate) fn convert_ceil(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    let arg = if let InstructionData::Unary { arg, .. } = inst_data {
        *arg
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Ceil instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let mapped_arg = map_value(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Ceil: round up to nearest integer
    // In fixed-point: (value + (1 << shift) - 1) >> shift, then << shift
    let mask = (1i64 << shift_amount) - 1;
    let mask_const = builder.ins().iconst(target_type, mask);
    let added = builder.ins().iadd(mapped_arg, mask_const);
    let shift_const = builder.ins().iconst(target_type, shift_amount as i64);
    let rounded = builder.ins().sshr(added, shift_const);
    let new_result = builder.ins().ishl(rounded, shift_const);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Floor instruction.
pub(crate) fn convert_floor(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    let arg = if let InstructionData::Unary { arg, .. } = inst_data {
        *arg
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Floor instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let mapped_arg = map_value(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Floor: round down to nearest integer
    // In fixed-point: value >> shift, then << shift
    let shift_const = builder.ins().iconst(target_type, shift_amount as i64);
    let rounded = builder.ins().sshr(mapped_arg, shift_const);
    let new_result = builder.ins().ishl(rounded, shift_const);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Sqrt instruction.
///
/// Note: This is a placeholder. A full implementation would need to use
/// fixed-point square root algorithms (e.g., CORDIC or external function calls).
pub(crate) fn convert_sqrt(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    // TODO: Implement fixed-point square root
    // This may require:
    // 1. External function call to sqrt implementation
    // 2. CORDIC algorithm
    // 3. Newton-Raphson method
    // For now, return an error to indicate it's not implemented
    return Err(GlslError::new(
        ErrorCode::E0301,
        "Sqrt conversion not yet implemented - requires fixed-point sqrt algorithm",
    ));
}

