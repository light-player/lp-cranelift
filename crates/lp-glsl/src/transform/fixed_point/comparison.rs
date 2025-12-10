//! Comparison operation converters (cmp, max, min).

use crate::error::GlslError;
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{
    Function, Inst, InstBuilder, InstructionData, Value,
    condcodes::{FloatCC, IntCC},
};

/// Convert Fcmp to Icmp with appropriate condition code
pub(super) fn convert_fcmp(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::FloatCompare {
        opcode: _,
        cond,
        args,
    } = inst_data
    {
        let arg1 = args[0];
        let arg2 = args[1];
        let cond = *cond;
        let old_result = func.dfg.first_result(inst);

        // Look up arguments in value_map
        let arg1_mapped = *value_map.get(&arg1).unwrap_or(&arg1);
        let arg2_mapped = *value_map.get(&arg2).unwrap_or(&arg2);

        // Convert FloatCC to IntCC
        let int_cond = match cond {
            FloatCC::Equal => IntCC::Equal,
            FloatCC::NotEqual => IntCC::NotEqual,
            FloatCC::LessThan => IntCC::SignedLessThan,
            FloatCC::LessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::GreaterThan => IntCC::SignedGreaterThan,
            FloatCC::GreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
            // For unordered/ordered: no NaN in fixed-point
            FloatCC::Ordered => IntCC::Equal, // Always true, use a == a
            FloatCC::Unordered => IntCC::NotEqual, // Always false, use a != a
            FloatCC::OrderedNotEqual => IntCC::NotEqual,
            FloatCC::UnorderedOrEqual => IntCC::Equal,
            FloatCC::UnorderedOrLessThan => IntCC::SignedLessThan,
            FloatCC::UnorderedOrLessThanOrEqual => IntCC::SignedLessThanOrEqual,
            FloatCC::UnorderedOrGreaterThan => IntCC::SignedGreaterThan,
            FloatCC::UnorderedOrGreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
        };

        // Create new icmp instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().icmp(int_cond, arg1_mapped, arg2_mapped);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Fmax to integer max using icmp + select
pub(super) fn convert_fmax(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let old_result = func.dfg.first_result(inst);

        // Create new max using comparison: max(a, b) = (a >= b) ? a : b
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let cmp = cursor
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, arg1, arg2);
        let new_result = cursor.ins().select(cmp, arg1, arg2);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Fmin to integer min using icmp + select
pub(super) fn convert_fmin(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let old_result = func.dfg.first_result(inst);

        // Create new min using comparison: min(a, b) = (a <= b) ? a : b
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let cmp = cursor.ins().icmp(IntCC::SignedLessThanOrEqual, arg1, arg2);
        let new_result = cursor.ins().select(cmp, arg1, arg2);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}
