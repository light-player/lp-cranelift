//! Memory operation converters (load, store).

use crate::error::GlslError;
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

/// Convert Load with F32 type to Load with I32/I64 type
pub(super) fn convert_load(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];

    // Check if this is a load of F32 type
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return Ok(()); // Not an F32 load, skip
    }

    if let InstructionData::Load {
        opcode: _,
        flags,
        offset,
        arg,
    } = inst_data
    {
        let addr = *arg;
        let flags = *flags;
        let offset = *offset;
        let target_type = format.cranelift_type();

        // Create new load instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().load(target_type, flags, addr, offset);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Store with F32 type to Store with I32/I64 type
pub(super) fn convert_store(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    _value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];

    if let InstructionData::Store {
        opcode: _,
        flags,
        offset,
        args,
    } = inst_data
    {
        let addr = args[0];
        let value = args[1];

        // Check if we're storing an F32 value
        if func.dfg.value_type(value) != cranelift_codegen::ir::types::F32 {
            return Ok(()); // Not an F32 store, skip
        }

        let flags = *flags;
        let offset = *offset;

        // Create new store instruction (store doesn't have a result)
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        cursor.ins().store(flags, value, addr, offset);

        // Remove old instruction
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}
