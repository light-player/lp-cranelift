//! Main transformation orchestration and signature conversion.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as ValueMap, format, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::HashMap as ValueMap, format, vec::Vec};

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Block, Function, Inst, Opcode, Value};

use super::arithmetic::{
    convert_fabs, convert_fadd, convert_fdiv, convert_fmul, convert_fneg, convert_fsub,
};
use super::calls::convert_call;
use super::comparison::{convert_fcmp, convert_fmax, convert_fmin};
use super::constants::convert_f32const;
use super::control::convert_select;
use super::conversions::{convert_fcvt_from_sint, convert_fcvt_from_uint};
use super::math::{convert_ceil, convert_floor, convert_sqrt};
use super::memory::{convert_load, convert_store};

/// Command returned by instruction visitor functions to control traversal.
pub(super) enum WalkCommand {
    /// Continue walking to the next instruction.
    Continue,
    /// Revisit the current instruction (because it was replaced and the new
    /// instruction may also need conversion).
    Revisit,
}

/// Walk forward through all instructions in all blocks, calling `f` for each.
///
/// This walks blocks in layout order and instructions within each block in
/// forward order. This ensures we process definitions before uses, which is
/// important for building the value_map incrementally.
fn forward_walk(func: &mut Function, mut f: impl FnMut(&mut Function, Block, Inst) -> WalkCommand) {
    let mut pos = FuncCursor::new(func);

    // Walk through all blocks in layout order
    while let Some(block) = pos.next_block() {
        // Position at first instruction of block
        pos.goto_first_inst(block);

        // Walk through all instructions in this block
        loop {
            let inst = match pos.current_inst() {
                Some(inst) => inst,
                None => break, // No more instructions in this block
            };

            let prev_pos = pos.position();
            match f(pos.func, block, inst) {
                WalkCommand::Continue => {
                    // Advance to next instruction
                    pos.next_inst();
                }
                WalkCommand::Revisit => {
                    // Revisit this instruction - reset position
                    pos.set_position(prev_pos);
                }
            }
        }
    }
}

/// Convert all float operations in a function to fixed-point.
///
/// This pass:
/// 1. Converts function signature (F32 → I32/I64)
/// 2. Replaces block parameters and builds initial value_map
/// 3. Single-pass visitor transformation
/// 4. Verifies no F32 values remain
/// 5. Verifies the function is still valid
pub fn convert_floats_to_fixed(
    func: &mut Function,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // 1. Convert signature (F32 params/returns → I32/I64)
    convert_signature(func, format);

    // 2. Replace block parameters and build initial value_map
    let mut value_map = replace_block_params(func, format);

    // 3. Single-pass visitor transformation
    forward_walk(func, |func, _block, inst| {
        convert_instruction_visitor(func, inst, format, &mut value_map)
    });

    // 4. Verify no F32 values remain
    verify_no_f32_values(func)?;

    // 5. Verify function is still valid
    if let Err(errors) = cranelift_codegen::verify_function(
        func,
        &cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()),
    ) {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "verification failed after fixed-point transformation: {}",
                errors
            ),
        ));
    }

    Ok(())
}

/// Verify that no F32 values remain in the function after transformation
fn verify_no_f32_values(func: &Function) -> Result<(), GlslError> {
    verify_signature(func)?;
    verify_block_params(func)?;
    verify_instructions(func)?;
    verify_jump_tables(func)?;
    Ok(())
}

/// Verify function signature has no F32 types
fn verify_signature(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    // Check parameters
    for (idx, param) in func.signature.params.iter().enumerate() {
        if param.value_type == types::F32 {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "F32 parameter still present after fixed-point transformation: param[{}] has type F32",
                    idx
                ),
            ));
        }
    }

    // Check return types
    for (idx, ret) in func.signature.returns.iter().enumerate() {
        if ret.value_type == types::F32 {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "F32 return type still present after fixed-point transformation: return[{}] has type F32",
                    idx
                ),
            ));
        }
    }

    Ok(())
}

/// Verify block parameters have no F32 types
fn verify_block_params(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    for block in func.layout.blocks() {
        for (idx, &param) in func.dfg.block_params(block).iter().enumerate() {
            if func.dfg.value_type(param) == types::F32 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "F32 block parameter still present after fixed-point transformation: block = `{}`, param[{}] = `{}` has type F32",
                        block, idx, param
                    ),
                ));
            }
        }
    }

    Ok(())
}

/// Verify all instructions have no F32 values
fn verify_instructions(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            // Skip instructions that have been converted (no longer F32 opcodes)
            // This avoids false positives from old result values that are aliased
            let opcode = func.dfg.insts[inst].opcode();
            if opcode == cranelift_codegen::ir::Opcode::F32const {
                // This should have been converted - if we see it, something went wrong
                // But don't check results here as they may be aliased
                continue;
            }

            // Check instruction results
            // Resolve aliases to check the actual type of values
            for &result in func.dfg.inst_results(inst) {
                let resolved_result = func.dfg.resolve_aliases(result);
                if func.dfg.value_type(resolved_result) == types::F32 {
                    return Err(GlslError::new(
                        ErrorCode::E0301,
                        format!(
                            "F32 result still present after fixed-point transformation: block = `{}`, inst = `{}`, result = `{}` (resolved: `{}`)",
                            block,
                            func.dfg.display_inst(inst),
                            result,
                            resolved_result
                        ),
                    ));
                }
            }

            // Check instruction operands
            for (idx, &arg) in func.dfg.inst_args(inst).iter().enumerate() {
                if func.dfg.value_type(arg) == types::F32 {
                    return Err(GlslError::new(
                        ErrorCode::E0301,
                        format!(
                            "F32 operand still present after fixed-point transformation: block = `{}`, inst = `{}`, operand[{}] = `{}` (type = F32)",
                            block,
                            func.dfg.display_inst(inst),
                            idx,
                            arg
                        ),
                    ));
                }
            }

            // Check branch arguments (values passed to blocks)
            for branch in func.dfg.insts[inst]
                .branch_destination(&func.dfg.jump_tables, &func.dfg.exception_tables)
            {
                for arg in branch.args(&func.dfg.value_lists) {
                    if let Some(val) = arg.as_value() {
                        if func.dfg.value_type(val) == types::F32 {
                            return Err(GlslError::new(
                                ErrorCode::E0301,
                                format!(
                                    "F32 value passed as branch argument: block = `{}`, inst = `{}`, value = `{}`",
                                    block,
                                    func.dfg.display_inst(inst),
                                    val
                                ),
                            ));
                        }
                    }
                }
            }

            // Check exception table contexts
            if let Some(et) = func.dfg.insts[inst].exception_table() {
                use cranelift_codegen::ir::ExceptionTableItem;
                for item in func.dfg.exception_tables[et].items() {
                    if let ExceptionTableItem::Context(ctx) = item {
                        if func.dfg.value_type(ctx) == types::F32 {
                            return Err(GlslError::new(
                                ErrorCode::E0301,
                                format!(
                                    "F32 value in exception table: block = `{}`, inst = `{}`, value = `{}`",
                                    block,
                                    func.dfg.display_inst(inst),
                                    ctx
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Verify jump tables have no F32 values
fn verify_jump_tables(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    for jump_table in func.dfg.jump_tables.values() {
        for branch in jump_table.all_branches() {
            for arg in branch.args(&func.dfg.value_lists) {
                if let Some(val) = arg.as_value() {
                    if func.dfg.value_type(val) == types::F32 {
                        return Err(GlslError::new(
                            ErrorCode::E0301,
                            format!("F32 value in jump table branch: value = `{}`", val),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn verify_no_f32_values_for_testing(func: &Function) -> Result<(), GlslError> {
    verify_no_f32_values(func)
}

/// Convert function signature: F32 params/returns → I32/I64
fn convert_signature(func: &mut Function, format: FixedPointFormat) {
    let target_type = format.cranelift_type();

    // Convert parameter types
    for param in &mut func.signature.params {
        if param.value_type == cranelift_codegen::ir::types::F32 {
            param.value_type = target_type;
        }
    }

    // Convert return types
    for ret in &mut func.signature.returns {
        if ret.value_type == cranelift_codegen::ir::types::F32 {
            ret.value_type = target_type;
        }
    }
}

/// Replace F32 block parameters with I32/I64 ones and build initial value_map.
///
/// This must be called BEFORE instruction conversion, as instructions
/// reference block parameters and need the mapped values.
///
/// Handles:
/// - Entry block parameters (function parameters)
/// - Block parameters from branches
/// - Multiple blocks with F32 parameters
///
/// Returns the value_map with block parameter mappings.
fn replace_block_params(func: &mut Function, format: FixedPointFormat) -> ValueMap<Value, Value> {
    let mut value_map = ValueMap::new();
    let target_type = format.cranelift_type();

    // Collect blocks and their F32 parameters first to avoid borrow conflicts
    let mut blocks_with_f32_params: Vec<(Block, Vec<Value>)> = Vec::new();
    for block in func.layout.blocks() {
        let params: Vec<Value> = func.dfg.block_params(block).to_vec();
        let f32_params: Vec<Value> = params
            .iter()
            .filter(|&&param| func.dfg.value_type(param) == cranelift_codegen::ir::types::F32)
            .copied()
            .collect();
        if !f32_params.is_empty() {
            blocks_with_f32_params.push((block, f32_params));
        }
    }

    // Replace F32 block parameters with I32/I64 ones
    // This updates the parameter type in the block's parameter list.
    // The old parameter value is detached but still exists in DFG.
    // We map it to the new parameter so all uses get updated via value_map.
    for (_block, f32_params) in blocks_with_f32_params {
        for old_param in f32_params {
            // Replace the F32 parameter with a new I32/I64 parameter
            // This updates the block's parameter list
            let new_param = func.dfg.replace_block_param(old_param, target_type);

            // Map old parameter to new one
            // All instructions using old_param will be updated via value_map
            value_map.insert(old_param, new_param);
        }
    }

    value_map
}

/// Visitor function for converting a single instruction.
/// Returns WalkCommand to control traversal.
fn convert_instruction_visitor(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let opcode = func.dfg.insts[inst].opcode();

    match opcode {
        Opcode::F32const => convert_f32const(func, inst, format, value_map),
        Opcode::Fadd => convert_fadd(func, inst, format, value_map),
        Opcode::Fsub => convert_fsub(func, inst, format, value_map),
        Opcode::Fmul => convert_fmul(func, inst, format, value_map),
        Opcode::Fdiv => convert_fdiv(func, inst, format, value_map),
        Opcode::Fneg => convert_fneg(func, inst, format, value_map),
        Opcode::Fabs => convert_fabs(func, inst, format, value_map),
        Opcode::Fcmp => convert_fcmp(func, inst, format, value_map),
        Opcode::Fmax => convert_fmax(func, inst, format, value_map),
        Opcode::Fmin => convert_fmin(func, inst, format, value_map),
        Opcode::Sqrt => {
            // Check if result is F32
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            if has_f32_result {
                convert_sqrt(func, inst, format, value_map)
            } else {
                WalkCommand::Continue
            }
        }
        Opcode::Ceil => {
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            if has_f32_result {
                convert_ceil(func, inst, format, value_map)
            } else {
                WalkCommand::Continue
            }
        }
        Opcode::Floor => {
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            if has_f32_result {
                convert_floor(func, inst, format, value_map)
            } else {
                WalkCommand::Continue
            }
        }
        Opcode::Select => {
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            if has_f32_result {
                convert_select(func, inst, format, value_map)
            } else {
                WalkCommand::Continue
            }
        }
        Opcode::Bitcast => {
            // Bitcast can convert F32 to I32/I64 (which we use for block parameters)
            // If it's converting F32 to our target type, it's fine (already correct)
            // Otherwise, if it has F32 operands/results, we need to handle it
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            let has_f32_operands = func
                .dfg
                .inst_args(inst)
                .iter()
                .any(|&arg| func.dfg.value_type(arg) == cranelift_codegen::ir::types::F32);
            let needs_conversion = has_f32_result || has_f32_operands;

            if needs_conversion {
                let target_type = format.cranelift_type();
                let result_type = func.dfg.value_type(func.dfg.first_result(inst));
                // Check if this is a bitcast from F32 to our target type (already correct)
                if result_type == target_type {
                    // This bitcast is converting F32 -> I32/I64, which is what we want
                    // Map the operand through value_map, but don't convert the instruction itself
                    if let Some(arg) = func.dfg.inst_args(inst).first() {
                        let mapped_arg = *value_map.get(arg).unwrap_or(arg);
                        if mapped_arg != *arg {
                            // Update the instruction to use the mapped operand
                            func.dfg.inst_args_mut(inst)[0] = mapped_arg;
                        }
                    }
                }
                // Otherwise, bitcast with F32 that doesn't produce our target type will be caught by verification
            }
            WalkCommand::Continue
        }
        Opcode::Load => convert_load(func, inst, format, value_map),
        Opcode::Store => convert_store(func, inst, format, value_map),
        Opcode::Call => convert_call(func, inst, format, value_map),
        Opcode::FcvtFromSint => {
            // Check if result is F32
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            if has_f32_result {
                convert_fcvt_from_sint(func, inst, format, value_map)
            } else {
                WalkCommand::Continue
            }
        }
        Opcode::FcvtFromUint => {
            // Check if result is F32
            let has_f32_result = func
                .dfg
                .inst_results(inst)
                .iter()
                .any(|&r| func.dfg.value_type(r) == cranelift_codegen::ir::types::F32);
            if has_f32_result {
                convert_fcvt_from_uint(func, inst, format, value_map)
            } else {
                WalkCommand::Continue
            }
        }
        Opcode::Jump | Opcode::Brif | Opcode::BrTable => {
            // Branch instructions pass arguments to destination blocks
            // Map all values (including branch arguments) through value_map
            // We need to handle this carefully to avoid borrow conflicts

            // Get mutable access to the instruction and related structures
            let dfg = &mut func.dfg;
            let inst_data = &mut dfg.insts[inst];
            let value_lists = &mut dfg.value_lists;
            let jump_tables = &mut dfg.jump_tables;
            let exception_tables = &mut dfg.exception_tables;

            // Map all values in the instruction (operands and branch arguments)
            inst_data.map_values(value_lists, jump_tables, exception_tables, |val| {
                *value_map.get(&val).unwrap_or(&val)
            });
            WalkCommand::Continue
        }
        Opcode::Return => {
            // Return doesn't need conversion - it just passes values through
            // In the single-pass approach, operands should already be mapped through value_map
            // Map operands through value_map to ensure we use converted values
            let args = func.dfg.inst_args(inst).to_vec();
            for (idx, &arg) in args.iter().enumerate() {
                // Check value_map for the original arg first
                if let Some(&mapped_arg) = value_map.get(&arg) {
                    if mapped_arg != arg {
                        func.dfg.inst_args_mut(inst)[idx] = mapped_arg;
                    }
                } else {
                    // Resolve aliases and check again
                    let resolved_arg = func.dfg.resolve_aliases(arg);
                    if resolved_arg != arg {
                        if let Some(&mapped_arg) = value_map.get(&resolved_arg) {
                            func.dfg.inst_args_mut(inst)[idx] = mapped_arg;
                        } else {
                            // If resolved arg is different but not in map, use resolved arg
                            func.dfg.inst_args_mut(inst)[idx] = resolved_arg;
                        }
                    }
                }
            }
            WalkCommand::Continue
        }
        _ => WalkCommand::Continue,
    }
}
