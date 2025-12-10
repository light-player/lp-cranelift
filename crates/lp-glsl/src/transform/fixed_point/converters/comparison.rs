//! Comparison operation conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

use cranelift_codegen::ir::{
    Function, Inst, InstBuilder, InstructionData, Value,
    condcodes::{FloatCC, IntCC},
};
use cranelift_frontend::FunctionBuilder;

use super::super::rewrite::map_value;

/// Convert Fcmp to icmp.
pub(crate) fn convert_fcmp(
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

    if let InstructionData::FloatCompare { cond, args, .. } = inst_data {
        // Map operands
        let arg1 = map_value(value_map, args[0]);
        let arg2 = map_value(value_map, args[1]);

        // Convert float condition to integer condition
        let int_cond = match cond {
            FloatCC::Equal => IntCC::Equal,
            FloatCC::NotEqual => IntCC::NotEqual,
            FloatCC::LessThan => IntCC::SignedLessThan,
            FloatCC::LessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::GreaterThan => IntCC::SignedGreaterThan,
            FloatCC::GreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
            FloatCC::Ordered => IntCC::Equal,      // Approximate
            FloatCC::Unordered => IntCC::NotEqual, // Approximate
            // Handle other conditions
            FloatCC::OrderedNotEqual => IntCC::NotEqual,
            FloatCC::UnorderedOrEqual => IntCC::Equal,
            FloatCC::UnorderedOrLessThan => IntCC::SignedLessThan,
            FloatCC::UnorderedOrLessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::UnorderedOrGreaterThan => IntCC::SignedGreaterThan,
            FloatCC::UnorderedOrGreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
        };

        // Emit icmp
        let new_result = builder.ins().icmp(int_cond, arg1, arg2);

        // Map result
        let old_result = old_func.dfg.first_result(old_inst);
        value_map.insert(old_result, new_result);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fcmp instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert Fmax to select with comparison.
pub(crate) fn convert_fmax(
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

    if let InstructionData::Binary { args, .. } = inst_data {
        let arg1 = map_value(value_map, args[0]);
        let arg2 = map_value(value_map, args[1]);

        // Compare and select maximum
        let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, arg1, arg2);
        let new_result = builder.ins().select(cmp, arg1, arg2);

        let old_result = old_func.dfg.first_result(old_inst);
        value_map.insert(old_result, new_result);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fmax instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert Fmin to select with comparison.
pub(crate) fn convert_fmin(
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

    if let InstructionData::Binary { args, .. } = inst_data {
        let arg1 = map_value(value_map, args[0]);
        let arg2 = map_value(value_map, args[1]);

        // Compare and select minimum
        let cmp = builder.ins().icmp(IntCC::SignedLessThan, arg1, arg2);
        let new_result = builder.ins().select(cmp, arg1, arg2);

        let old_result = old_func.dfg.first_result(old_inst);
        value_map.insert(old_result, new_result);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fmin instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

