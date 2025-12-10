//! Constant conversion functions.

use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

use super::transform::WalkCommand;
use super::types::{float_to_fixed16x16, float_to_fixed32x32};

/// Convert F32const to iconst with fixed-point value
pub(super) fn convert_f32const(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    // Get the float constant value
    let inst_data = &func.dfg.insts[inst];

    // Verify this is actually an F32const instruction
    let opcode = inst_data.opcode();
    if opcode != cranelift_codegen::ir::Opcode::F32const {
        // This should never happen - opcode should be matched before calling this function
        return WalkCommand::Continue;
    }

    // Check if result is F32 (should always be true for F32const, but check anyway)
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    // Extract the float value from UnaryIeee32 format
    let f32_value = match inst_data {
        InstructionData::UnaryIeee32 { opcode: _, imm } => f32::from_bits(imm.bits()),
        _ => {
            // This should never happen for F32const opcode
            // If it does, something is wrong with our opcode matching
            panic!(
                "convert_f32const called on instruction with wrong InstructionData variant: opcode={:?}, data={:?}",
                opcode, inst_data
            );
        }
    };

    // Convert to fixed-point
    let fixed_value = match format {
        FixedPointFormat::Fixed16x16 => float_to_fixed16x16(f32_value) as i64,
        FixedPointFormat::Fixed32x32 => float_to_fixed32x32(f32_value),
    };

    // Replace instruction in-place
    let target_type = format.cranelift_type();

    // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
    // According to ReplaceBuilder docs: "If the old instruction still has result values
    // attached, it is assumed that the new instruction produces the same number and types
    // of results. The old result values are preserved."
    // We want new results with a different type (F32 -> I32/I64), so we must detach first.
    func.dfg.detach_inst_results(inst);

    // Use replace() which overwrites the instruction data and creates new result values
    // This replaces the f32const instruction with an iconst instruction
    // Since we detached results, it will create new ones
    let new_result = func.dfg.replace(inst).iconst(target_type, fixed_value);

    // Add to value_map immediately - this maps old F32 result to new I32/I64 result
    // All uses of old_result will be redirected to new_result via value_map during forward_walk
    // Note: We do NOT use change_to_alias here because it's designed for same-type aliasing,
    // and we're converting from F32 to I32/I64. The value_map mechanism handles cross-type
    // value replacement correctly.
    value_map.insert(old_result, new_result);

    WalkCommand::Continue
}
