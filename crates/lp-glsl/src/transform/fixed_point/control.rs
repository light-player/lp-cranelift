//! Control flow operation converters (select).

use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

use super::transform::WalkCommand;

/// Convert Select to handle float operands
pub(super) fn convert_select(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Ternary { opcode: _, args } = inst_data {
        let cond = args[0];
        let arg1 = args[1];
        let arg2 = args[2];

        // Check if result is F32 type - only convert if result is F32
        let old_result = func.dfg.first_result(inst);
        let result_type = func.dfg.value_type(old_result);

        if result_type != cranelift_codegen::ir::types::F32 {
            // Result is not F32, no conversion needed
            return WalkCommand::Continue;
        }

        // Map operands through value_map FIRST
        let val1 = *value_map.get(&arg1).unwrap_or(&arg1);
        let val2 = *value_map.get(&arg2).unwrap_or(&arg2);

        // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
        // We're changing the result type (F32 -> I32/I64), so we must detach first.
        func.dfg.detach_inst_results(inst);

        // Replace instruction in-place
        let new_result = func.dfg.replace(inst).select(cond, val1, val2);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}
