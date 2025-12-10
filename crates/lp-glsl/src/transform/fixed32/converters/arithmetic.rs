//! Arithmetic operation conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::{
    Function, Inst, InstBuilder, InstructionData, Value, condcodes::IntCC,
};
use cranelift_frontend::FunctionBuilder;

use super::super::rewrite::map_value;

/// Convert Fadd to iadd.
pub(crate) fn convert_fadd(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    // Get operands and map them
    let (arg1_old, arg2_old) = if let InstructionData::Binary { args, .. } = inst_data {
        (args[0], args[1])
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fadd instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let arg1 = map_value(value_map, arg1_old);
    let arg2 = map_value(value_map, arg2_old);

    // Emit iadd instruction
    let new_result = builder.ins().iadd(arg1, arg2);

    // Map old result to new result
    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fsub to isub.
pub(crate) fn convert_fsub(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    let (arg1_old, arg2_old) = if let InstructionData::Binary { args, .. } = inst_data {
        (args[0], args[1])
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fsub instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let arg1 = map_value(value_map, arg1_old);
    let arg2 = map_value(value_map, arg2_old);

    let new_result = builder.ins().isub(arg1, arg2);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fmul to imul with scaling.
pub(crate) fn convert_fmul(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    let (arg1_old, arg2_old) = if let InstructionData::Binary { args, .. } = inst_data {
        (args[0], args[1])
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fmul instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let arg1 = map_value(value_map, arg1_old);
    let arg2 = map_value(value_map, arg2_old);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Fixed-point multiplication: (a * b) >> shift_amount
    let mul_result = builder.ins().imul(arg1, arg2);

    // Right shift to scale back
    let shift_const = builder.ins().iconst(target_type, shift_amount as i64);
    let new_result = builder.ins().sshr(mul_result, shift_const);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fdiv to sdiv with scaling.
pub(crate) fn convert_fdiv(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    let (arg1_old, arg2_old) = if let InstructionData::Binary { args, .. } = inst_data {
        (args[0], args[1])
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fdiv instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let arg1 = map_value(value_map, arg1_old);
    let arg2 = map_value(value_map, arg2_old);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Fixed-point division: (a << shift_amount) / b
    // Left shift numerator first
    let shift_const = builder.ins().iconst(target_type, shift_amount as i64);
    let shifted_numerator = builder.ins().ishl(arg1, shift_const);

    // Divide
    let new_result = builder.ins().sdiv(shifted_numerator, arg2);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fneg to ineg.
pub(crate) fn convert_fneg(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    let arg = if let InstructionData::Unary { arg, .. } = inst_data {
        *arg
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fneg instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let mapped_arg = map_value(value_map, arg);

    let new_result = builder.ins().ineg(mapped_arg);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fabs using conditional select.
pub(crate) fn convert_fabs(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    let arg = if let InstructionData::Unary { arg, .. } = inst_data {
        *arg
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Fabs instruction has unexpected format: {:?}", inst_data),
        ));
    };

    let mapped_arg = map_value(value_map, arg);
    let target_type = format.cranelift_type();

    // Absolute value: if (arg < 0) then -arg else arg
    let zero = builder.ins().iconst(target_type, 0);
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, mapped_arg, zero);
    let negated = builder.ins().ineg(mapped_arg);
    let new_result = builder.ins().select(is_negative, negated, mapped_arg);

    let old_result = old_func.dfg.first_result(old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

