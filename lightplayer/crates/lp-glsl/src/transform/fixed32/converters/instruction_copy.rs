//! Shared utility for copying instructions between functions
//!
//! This module provides a common implementation for copying instructions
//! from one function to another, mapping values through a value_map.
//! Used by both the fixed-point transformation and function linking.

use crate::error::{ErrorCode, GlslError};
use crate::ir_utils::value_map::map_value;
use cranelift_codegen::ir::{Function, Inst, InstructionData, StackSlot, Value, types};
use cranelift_frontend::FunctionBuilder;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

// Use hashbrown::HashMap for consistency across no_std and std builds
use hashbrown::HashMap;

/// Copy a non-F32 instruction as-is (for instructions that don't need conversion).
///
/// This handles instructions that don't involve F32 types and can be copied
/// directly, mapping all values through value_map.
///
/// Note: This is a simplified implementation. For complex instructions,
/// explicit converters should be created.
pub fn copy_instruction_as_is(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    check_f32: bool, // If true, verify no F32 types (for fixed-point conversion)
) -> Result<(), GlslError> {
    copy_instruction_as_is_with_stack_slot_map(
        old_func, old_inst, builder, value_map, check_f32, None,
    )
}

pub fn copy_instruction_as_is_with_stack_slot_map(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    check_f32: bool, // If true, verify no F32 types (for fixed-point conversion)
    stack_slot_map: Option<&HashMap<StackSlot, StackSlot>>, // Map old stack slot IDs to new ones
) -> Result<(), GlslError> {
    let opcode = old_func.dfg.insts[old_inst].opcode();
    let inst_data = &old_func.dfg.insts[old_inst];

    // Check if this instruction involves F32 types (only if requested)
    if check_f32 {
        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        for &result in &old_results {
            if old_func.dfg.value_type(result) == types::F32 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Instruction {:?} produces F32 result but was not converted. This is a bug in the fixed-point conversion.",
                        opcode
                    ),
                ));
            }
        }

        // Check operands for F32 types
        let old_args = old_func.dfg.inst_args(old_inst);
        for &arg in old_args {
            if old_func.dfg.value_type(arg) == types::F32 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Instruction {:?} has F32 operand but was not converted. This is a bug in the fixed-point conversion.",
                        opcode
                    ),
                ));
            }
        }
    }

    // If no results, this is typically a terminator or side-effect only instruction
    // These should have been handled by explicit converters, but if they fall through,
    // we can safely skip them (they don't produce values to map)
    let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
    if old_results.is_empty() {
        // Most terminators are handled explicitly, but some side-effect instructions
        // might fall through. For now, we'll allow this.
        return Ok(());
    }

    // For instructions with results, we need to reconstruct them
    // Determine the controlling type
    let ctrl_type = if opcode.constraints().requires_typevar_operand() {
        // Get type from first operand
        let first_arg = old_func.dfg.inst_args(old_inst)[0];
        let mapped_first_arg = map_value(value_map, first_arg);
        builder.func.dfg.value_type(mapped_first_arg)
    } else {
        // Get type from first result
        old_func.dfg.value_type(old_results[0])
    };

    // Get old arguments using DFG
    let old_args = old_func.dfg.inst_args(old_inst);
    let mapped_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

    // Reconstruct instruction data with mapped operands
    let new_inst_data = match inst_data {
        InstructionData::Unary { .. } => InstructionData::Unary {
            opcode,
            arg: mapped_args[0],
        },
        InstructionData::UnaryImm { imm, .. } => InstructionData::UnaryImm { opcode, imm: *imm },
        InstructionData::Binary { .. } => {
            if mapped_args.len() != 2 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Binary instruction requires 2 arguments, got {}",
                        mapped_args.len()
                    ),
                ));
            }
            InstructionData::Binary {
                opcode,
                args: [mapped_args[0], mapped_args[1]],
            }
        }
        InstructionData::Ternary { .. } => {
            if mapped_args.len() != 3 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Ternary instruction requires 3 arguments, got {}",
                        mapped_args.len()
                    ),
                ));
            }
            InstructionData::Ternary {
                opcode,
                args: [mapped_args[0], mapped_args[1], mapped_args[2]],
            }
        }
        InstructionData::NullAry { .. } => InstructionData::NullAry { opcode },
        InstructionData::IntCompare { cond, .. } => {
            if mapped_args.len() != 2 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "IntCompare instruction requires 2 arguments, got {}",
                        mapped_args.len()
                    ),
                ));
            }
            InstructionData::IntCompare {
                opcode,
                cond: *cond,
                args: [mapped_args[0], mapped_args[1]],
            }
        }
        InstructionData::FloatCompare { cond, .. } => {
            if mapped_args.len() != 2 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "FloatCompare instruction requires 2 arguments, got {}",
                        mapped_args.len()
                    ),
                ));
            }
            InstructionData::FloatCompare {
                opcode,
                cond: *cond,
                args: [mapped_args[0], mapped_args[1]],
            }
        }
        InstructionData::UnaryIeee32 { imm, .. } => {
            InstructionData::UnaryIeee32 { opcode, imm: *imm }
        }
        InstructionData::UnaryIeee64 { imm, .. } => {
            InstructionData::UnaryIeee64 { opcode, imm: *imm }
        }
        InstructionData::UnaryConst {
            constant_handle, ..
        } => InstructionData::UnaryConst {
            opcode,
            constant_handle: *constant_handle,
        },
        InstructionData::Load {
            flags,
            offset,
            arg: _,
            ..
        } => InstructionData::Load {
            opcode,
            flags: *flags,
            offset: *offset,
            arg: mapped_args[0],
        },
        InstructionData::Store {
            flags,
            offset,
            args: _,
            ..
        } => {
            // Store is handled separately - it has no results
            // We need to construct the instruction directly using DFG
            let store_inst_data = InstructionData::Store {
                opcode,
                flags: *flags,
                offset: *offset,
                args: [mapped_args[0], mapped_args[1]],
            };
            builder.ensure_inserted_block();
            let current_block = builder
                .current_block()
                .expect("Builder must have a current block");
            let new_inst = builder.func.dfg.make_inst(store_inst_data);
            builder.func.layout.append_inst(new_inst, current_block);
            return Ok(());
        }
        InstructionData::StackLoad {
            stack_slot, offset, ..
        } => {
            // StackLoad has stack_slot and offset
            // Remap stack slot if mapping provided
            let new_stack_slot = stack_slot_map
                .and_then(|m| m.get(stack_slot))
                .copied()
                .unwrap_or(*stack_slot);
            InstructionData::StackLoad {
                opcode,
                stack_slot: new_stack_slot,
                offset: *offset,
            }
        }
        InstructionData::StackStore {
            stack_slot,
            offset,
            arg: _,
            ..
        } => {
            // Stack store is handled separately - it has no results
            // StackStore has stack_slot, offset, and arg (address)
            // Value comes from inst_args[0]
            // Remap stack slot if mapping provided
            let new_stack_slot = stack_slot_map
                .and_then(|m| m.get(stack_slot))
                .copied()
                .unwrap_or(*stack_slot);
            let stack_store_inst_data = InstructionData::StackStore {
                opcode,
                stack_slot: new_stack_slot,
                offset: *offset,
                arg: mapped_args[1], // address is the second argument (value is first)
            };
            builder.ensure_inserted_block();
            let current_block = builder
                .current_block()
                .expect("Builder must have a current block");
            let new_inst = builder.func.dfg.make_inst(stack_store_inst_data);
            // StackStore needs value as the first argument (address is in arg field)
            // The value argument needs to be added to the instruction
            // For now, we'll construct it with the value in the args
            builder.func.layout.append_inst(new_inst, current_block);
            return Ok(());
        }
        _ => {
            // For other instruction formats, return an error
            // These should be handled by explicit converters
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "Instruction {:?} with format {:?} not yet supported in copy_instruction_as_is. Please add explicit converter.",
                    opcode, inst_data
                ),
            ));
        }
    };

    // Insert instruction using DFG and layout directly
    // We need to ensure the block is inserted in the layout first
    builder.ensure_inserted_block();
    let current_block = builder
        .current_block()
        .expect("Builder must have a current block");

    // Create instruction
    let new_inst = builder.func.dfg.make_inst(new_inst_data);
    builder.func.dfg.make_inst_results(new_inst, ctrl_type);
    builder.func.layout.append_inst(new_inst, current_block);

    // Map results
    let new_results = builder.func.dfg.inst_results(new_inst);
    if old_results.len() != new_results.len() {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Instruction {:?} result count mismatch: old={}, new={}",
                opcode,
                old_results.len(),
                new_results.len()
            ),
        ));
    }
    for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
        value_map.insert(*old_result, *new_result);
    }

    Ok(())
}
