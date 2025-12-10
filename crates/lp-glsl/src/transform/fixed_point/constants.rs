//! Constant conversion functions.

use crate::error::GlslError;
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

use super::types::{float_to_fixed16x16, float_to_fixed32x32};

/// Convert F32const to iconst with fixed-point value
pub(super) fn convert_f32const(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    // Get the float constant value
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::UnaryIeee32 { opcode: _, imm } = inst_data {
        let f32_value = f32::from_bits(imm.bits());
        let old_result = func.dfg.first_result(inst);

        // Convert to fixed-point
        let fixed_value = match format {
            FixedPointFormat::Fixed16x16 => float_to_fixed16x16(f32_value) as i64,
            FixedPointFormat::Fixed32x32 => float_to_fixed32x32(f32_value),
        };

        // Create new iconst instruction with cursor
        let target_type = format.cranelift_type();
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().iconst(target_type, fixed_value);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach old result and remove the old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}
