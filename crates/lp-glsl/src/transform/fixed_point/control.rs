//! Control flow operation converters (select).

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

/// Convert Select to handle float operands
pub(super) fn convert_select(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Ternary { opcode: _, args } = inst_data {
        let cond = args[0];
        let arg1 = args[1];
        let arg2 = args[2];

        // Check if operands are F32 type
        let arg1_type = func.dfg.value_type(arg1);
        let arg2_type = func.dfg.value_type(arg2);
        let target_type = format.cranelift_type();

        // If both operands are already converted (not F32), no conversion needed
        if arg1_type != cranelift_codegen::ir::types::F32
            && arg2_type != cranelift_codegen::ir::types::F32
        {
            return Ok(());
        }

        let old_result = func.dfg.first_result(inst);

        // Get mapped values and check types before creating cursor
        let val1 = *value_map.get(&arg1).unwrap_or(&arg1);
        let val2 = *value_map.get(&arg2).unwrap_or(&arg2);

        // Check types before creating cursor to avoid borrow conflicts
        let val1_type = func.dfg.value_type(val1);
        let val2_type = func.dfg.value_type(val2);

        // Ensure both are the target type (they should be, but handle edge cases)
        if val1_type == cranelift_codegen::ir::types::F32
            || val2_type == cranelift_codegen::ir::types::F32
        {
            return Err(GlslError::new(
                ErrorCode::E0301,
                "select operand still F32 after conversion",
            ));
        }

        if val1_type != target_type || val2_type != target_type {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!("select operand has unexpected type after conversion"),
            ));
        }

        let mut cursor = FuncCursor::new(func).at_inst(inst);

        // Create new select instruction with converted operands
        let new_result = cursor.ins().select(cond, val1, val2);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}
