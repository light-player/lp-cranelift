//! Memory operation converters (load, store).

use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

use super::transform::WalkCommand;

/// Convert Load with F32 type to Load with I32/I64 type
pub(super) fn convert_load(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    // Check if this is a load of F32 type
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not an F32 load, skip
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

        // Map address through value_map FIRST (in case it's a converted value)
        let mapped_addr = *value_map.get(&addr).unwrap_or(&addr);

        // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
        // We're changing the result type (F32 -> I32/I64), so we must detach first.
        func.dfg.detach_inst_results(inst);

        // Replace instruction in-place
        let new_result = func
            .dfg
            .replace(inst)
            .load(target_type, flags, mapped_addr, offset);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}

/// Convert Store with F32 type to Store with I32/I64 type
pub(super) fn convert_store(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    if let InstructionData::Store {
        opcode: _,
        flags,
        offset,
        args,
    } = inst_data
    {
        // Map address operand through value_map FIRST (always needed)
        let addr = args[0];
        let mapped_addr = *value_map.get(&addr).unwrap_or(&addr);
        let value = args[1];

        // Check if we're storing an F32 value
        if func.dfg.value_type(value) != cranelift_codegen::ir::types::F32 {
            // Not an F32 store, but still need to map address
            // Update instruction to use mapped address if it changed
            if mapped_addr != addr {
                func.dfg.inst_args_mut(inst)[0] = mapped_addr;
            }
            return WalkCommand::Continue;
        }

        // Map value operand through value_map
        let fixed_value = *value_map.get(&value).unwrap_or(&value);

        // Verify the mapped value has the correct type
        let target_type = format.cranelift_type();
        if func.dfg.value_type(fixed_value) == cranelift_codegen::ir::types::F32 {
            // Still F32 - this is an error, but let verification catch it
            return WalkCommand::Continue;
        }

        let flags = *flags;
        let offset = *offset;

        // Replace instruction in-place
        func.dfg
            .replace(inst)
            .store(flags, fixed_value, mapped_addr, offset);
    }

    WalkCommand::Continue
}
