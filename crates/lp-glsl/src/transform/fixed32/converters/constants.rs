//! Constant conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::{FixedPointFormat, float_to_fixed16x16};

use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};
use cranelift_frontend::FunctionBuilder;

/// Convert F32const to iconst with fixed-point value.
pub(crate) fn convert_f32const(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Get the float constant value
    let inst_data = &old_func.dfg.insts[old_inst];

    // Extract the float value from UnaryIeee32 format
    let f32_value = match inst_data {
        InstructionData::UnaryIeee32 { opcode: _, imm } => f32::from_bits(imm.bits()),
        _ => {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "F32const instruction has unexpected format: {:?}",
                    inst_data
                ),
            ));
        }
    };

    // Convert to fixed-point
    let target_type = format.cranelift_type();
    let fixed_value = match format {
        FixedPointFormat::Fixed16x16 => float_to_fixed16x16(f32_value) as i64,
    };

    // Emit iconst instruction
    let new_result = builder.ins().iconst(target_type, fixed_value);

    // Map old result to new result
    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}
