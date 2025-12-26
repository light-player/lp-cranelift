//! Constant conversion functions.

use crate::error::GlslError;
use crate::backend::transform::fixed32::types::{FixedPointFormat, float_to_fixed16x16};

use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

use super::{get_first_result, unexpected_format_error};

/// Convert F32const to iconst with fixed-point value.
pub(crate) fn convert_f32const(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Get the float constant value
    let inst_data = &old_func.dfg.insts[old_inst];

    // Extract the float value from UnaryIeee32 format
    let f32_value = match inst_data {
        InstructionData::UnaryIeee32 { opcode: _, imm } => f32::from_bits(imm.bits()),
        _ => {
            return Err(unexpected_format_error(old_func, old_inst, "UnaryIeee32"));
        }
    };

    // Convert to fixed-point
    let target_type = format.cranelift_type();
    let fixed_value = match format {
        FixedPointFormat::Fixed16x16 => float_to_fixed16x16(f32_value) as i64,
        FixedPointFormat::Fixed32x32 => {
            // Not fully implemented - use a placeholder
            (f32_value * 4294967296.0) as i64
        }
    };

    // Emit iconst instruction
    let new_result = builder.ins().iconst(target_type, fixed_value);

    // Map old result to new result
    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}
