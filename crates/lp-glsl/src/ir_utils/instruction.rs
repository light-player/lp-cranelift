//! Instruction extraction and validation utilities.

use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{Function, Inst, InstructionData, Value, types};
use cranelift_frontend::FunctionBuilder;

/// Extract binary operands from an instruction.
///
/// Returns an error if the instruction is not in Binary format.
pub fn extract_binary_operands(
    old_func: &Function,
    old_inst: Inst,
) -> Result<(Value, Value), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Binary { args, .. } = inst_data {
        Ok((args[0], args[1]))
    } else {
        Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Expected binary instruction format, got: {:?} (opcode: {:?})",
                inst_data,
                old_func.dfg.insts[old_inst].opcode()
            ),
        ))
    }
}

/// Extract unary operand from an instruction.
///
/// Returns an error if the instruction is not in Unary format.
pub fn extract_unary_operand(old_func: &Function, old_inst: Inst) -> Result<Value, GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Unary { arg, .. } = inst_data {
        Ok(*arg)
    } else {
        Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Expected unary instruction format, got: {:?} (opcode: {:?})",
                inst_data,
                old_func.dfg.insts[old_inst].opcode()
            ),
        ))
    }
}

/// Get the first result value from an instruction.
pub fn get_first_result(old_func: &Function, old_inst: Inst) -> Value {
    old_func.dfg.first_result(old_inst)
}

/// Verify that a value has the expected type after conversion.
///
/// Returns an error if the type doesn't match, providing context about
/// the instruction and expected vs actual types.
pub fn verify_converted_type(
    builder: &FunctionBuilder,
    value: Value,
    expected_type: types::Type,
    old_func: &Function,
    old_inst: Inst,
) -> Result<(), GlslError> {
    let actual_type = builder.func.dfg.value_type(value);
    if actual_type != expected_type {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Type mismatch after conversion: instruction {:?} (opcode: {:?}) expected type {:?}, got {:?}",
                old_inst,
                old_func.dfg.insts[old_inst].opcode(),
                expected_type,
                actual_type
            ),
        ));
    }
    Ok(())
}

/// Create an error for an unexpected instruction format.
///
/// Provides detailed context including the instruction, opcode, and actual format.
pub fn unexpected_format_error(
    old_func: &Function,
    old_inst: Inst,
    expected_format: &str,
) -> GlslError {
    let inst_data = &old_func.dfg.insts[old_inst];
    let opcode = inst_data.opcode();
    GlslError::new(
        ErrorCode::E0301,
        format!(
            "{} instruction has unexpected format: expected {}, got {:?} (opcode: {:?})",
            opcode, expected_format, inst_data, opcode
        ),
    )
}
