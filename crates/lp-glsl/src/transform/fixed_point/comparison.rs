//! Comparison operation converters (cmp, max, min).

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

use super::transform::WalkCommand;

/// Convert Fcmp to Icmp with appropriate condition code
pub(super) fn convert_fcmp(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
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

        // Map operands through value_map FIRST
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

        // Replace instruction in-place
        let new_result = func
            .dfg
            .replace(inst)
            .icmp(int_cond, arg1_mapped, arg2_mapped);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}

/// Convert Fmax to integer max using icmp + select
pub(super) fn convert_fmax(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let old_result = func.dfg.first_result(inst);

        // Map operands through value_map FIRST
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);

        // Create new max using comparison: max(a, b) = (a >= b) ? a : b
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let cmp = cursor
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, arg1, arg2);
        let new_result = cursor.ins().select(cmp, arg1, arg2);

        // Detach old instruction results
        cursor.func.dfg.detach_inst_results(inst);

        // Add to value_map immediately - this maps old F32 result to new I32/I64 result
        // All uses of old_result will be redirected to new_result via value_map during forward_walk
        // Note: We do NOT use change_to_alias here because it's designed for same-type aliasing,
        // and we're converting from F32 to I32/I64. The value_map mechanism handles cross-type
        // value replacement correctly.
        value_map.insert(old_result, new_result);

        // Replace old instruction with a harmless instruction (iconst 0)
        // The alias ensures correctness, this is just to clean up the instruction
        let target_type = cursor.func.dfg.value_type(new_result);
        cursor.func.dfg.replace(inst).iconst(target_type, 0);
    }

    WalkCommand::Continue
}

/// Convert Fmin to integer min using icmp + select
pub(super) fn convert_fmin(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let old_result = func.dfg.first_result(inst);

        // Map operands through value_map FIRST
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);

        // Create new min using comparison: min(a, b) = (a <= b) ? a : b
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let cmp = cursor.ins().icmp(IntCC::SignedLessThanOrEqual, arg1, arg2);
        let new_result = cursor.ins().select(cmp, arg1, arg2);

        // Detach old instruction results
        cursor.func.dfg.detach_inst_results(inst);

        // Add to value_map immediately - this maps old F32 result to new I32/I64 result
        // All uses of old_result will be redirected to new_result via value_map during forward_walk
        // Note: We do NOT use change_to_alias here because it's designed for same-type aliasing,
        // and we're converting from F32 to I32/I64. The value_map mechanism handles cross-type
        // value replacement correctly.
        value_map.insert(old_result, new_result);

        // Replace old instruction with a harmless instruction (iconst 0)
        // The alias ensures correctness, this is just to clean up the instruction
        let target_type = cursor.func.dfg.value_type(new_result);
        cursor.func.dfg.replace(inst).iconst(target_type, 0);
    }

    WalkCommand::Continue
}
