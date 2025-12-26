//! Instruction copying utilities.
//!
//! Provides utilities for copying instructions with configurable transformations.

use crate::error::GlslError;
use crate::ir_utils::value_map::map_value;
use cranelift_codegen::ir::{Block, FuncRef, Function, Inst, Value};
use cranelift_frontend::FunctionBuilder;
use cranelift_codegen::ir::InstBuilder;
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// Trait for transforming instructions during copying.
///
/// Implementations can:
/// - Return None to skip the instruction (for instructions that should be handled specially)
/// - Return Some(inst) to copy the instruction with transformations
pub trait InstructionTransformer {
    /// Transform a single instruction.
    ///
    /// Returns Ok(None) if the instruction was handled specially (e.g., by explicit converters).
    /// Returns Ok(Some(new_inst)) if the instruction was copied/transformed.
    /// Returns Err if there was an error.
    fn transform_instruction(
        &mut self,
        old_func: &Function,
        old_inst: Inst,
        builder: &mut FunctionBuilder,
        value_map: &mut HashMap<Value, Value>,
        block_map: &HashMap<Block, Block>,
    ) -> Result<Option<Inst>, GlslError>;
}

/// No-op transformer that copies instructions exactly as-is.
///
/// This is used for exact copying (e.g., during linking).
pub struct NoOpTransformer {
    /// Optional stack slot mapping for remapping stack slot references
    pub stack_slot_map: Option<HashMap<cranelift_codegen::ir::StackSlot, cranelift_codegen::ir::StackSlot>>,
    /// Optional function reference mapping for remapping FuncRefs
    pub func_ref_map: Option<HashMap<FuncRef, FuncRef>>,
}

impl InstructionTransformer for NoOpTransformer {
    fn transform_instruction(
        &mut self,
        old_func: &Function,
        old_inst: Inst,
        builder: &mut FunctionBuilder,
        value_map: &mut HashMap<Value, Value>,
        block_map: &HashMap<Block, Block>,
    ) -> Result<Option<Inst>, GlslError> {
        use cranelift_codegen::ir::{BlockArg, InstructionData, Opcode};
        use crate::ir_utils::value_map::map_value;

        let opcode = old_func.dfg.insts[old_inst].opcode();
        let inst_data = &old_func.dfg.insts[old_inst];

        // Handle control flow instructions explicitly to ensure block parameters are preserved
        match opcode {
            Opcode::Call => {
                // Handle call instruction with FuncRef remapping
                if let InstructionData::Call { func_ref, args, .. } = inst_data {
                    let new_func_ref = if let Some(ref func_ref_map) = self.func_ref_map {
                        *func_ref_map.get(func_ref).ok_or_else(|| {
                            GlslError::new(
                                crate::error::ErrorCode::E0400,
                                format!("Could not find new FuncRef for old FuncRef in call instruction"),
                            )
                        })?
                    } else {
                        *func_ref
                    };

                    let old_args = args.as_slice(&old_func.dfg.value_lists);
                    let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

                    let call_inst = builder.ins().call(new_func_ref, &new_args);

                    // Map return values
                    let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
                    let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

                    if old_results.len() != new_results.len() {
                        return Err(GlslError::new(
                            crate::error::ErrorCode::E0301,
                            format!(
                                "Call return value count mismatch: old={}, new={}",
                                old_results.len(),
                                new_results.len()
                            ),
                        ));
                    }

                    for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
                        value_map.insert(*old_result, *new_result);
                    }
                }
            }
            Opcode::Jump => {
                // Handle jump instruction
                // Block parameters are already ensured in the first pass of copy_instructions
                if let InstructionData::Jump { destination, .. } = inst_data {
                    let old_dest_block = destination.block(&old_func.dfg.value_lists);
                    let new_dest_block = block_map[&old_dest_block];

                    let old_args: Vec<Value> = destination
                        .args(&old_func.dfg.value_lists)
                        .filter_map(|arg| arg.as_value())
                        .collect();

                    // Map arguments - block parameters are already ensured in the first pass
                    let new_args: Vec<BlockArg> = old_args
                        .iter()
                        .map(|&v| map_value(value_map, v).into())
                        .collect();

                    builder.ins().jump(new_dest_block, &new_args);
                }
            }
            Opcode::Brif => {
                // Handle brif instruction
                // Block parameters are already ensured in the first pass of copy_instructions
                if let InstructionData::Brif {
                    arg,
                    blocks: [block_then_call, block_else_call],
                    ..
                } = inst_data {
                    let condition = map_value(value_map, *arg);

                    let old_then_block = block_then_call.block(&old_func.dfg.value_lists);
                    let old_else_block = block_else_call.block(&old_func.dfg.value_lists);

                    let new_then_block = block_map[&old_then_block];
                    let new_else_block = block_map[&old_else_block];

                    let old_then_args: Vec<Value> = block_then_call
                        .args(&old_func.dfg.value_lists)
                        .filter_map(|arg| arg.as_value())
                        .collect();
                    let old_else_args: Vec<Value> = block_else_call
                        .args(&old_func.dfg.value_lists)
                        .filter_map(|arg| arg.as_value())
                        .collect();

                    // Map arguments - block parameters are already ensured in the first pass
                    let new_then_args: Vec<BlockArg> = old_then_args
                        .iter()
                        .map(|&v| map_value(value_map, v).into())
                        .collect();
                    let new_else_args: Vec<BlockArg> = old_else_args
                        .iter()
                        .map(|&v| map_value(value_map, v).into())
                        .collect();

                    builder.ins().brif(
                        condition,
                        new_then_block,
                        &new_then_args,
                        new_else_block,
                        &new_else_args,
                    );
                }
            }
            Opcode::Return => {
                // Handle return instruction
                if let InstructionData::MultiAry { args, .. } = inst_data {
                    let old_args = args.as_slice(&old_func.dfg.value_lists);
                    let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();
                    builder.ins().return_(&new_args);
                }
            }
            Opcode::Select => {
                // Handle select instruction
                if let InstructionData::Ternary { args, .. } = inst_data {
                    let condition = map_value(value_map, args[0]);
                    let true_val = map_value(value_map, args[1]);
                    let false_val = map_value(value_map, args[2]);

                    let new_result = builder.ins().select(condition, true_val, false_val);
                    let old_result = old_func.dfg.first_result(old_inst);
                    value_map.insert(old_result, new_result);
                }
            }
            _ => {
                // For other instructions, use the existing copy_instruction_as_is utility
                use crate::transform::fixed32::converters;
                converters::copy_instruction_as_is_with_stack_slot_map(
                    old_func,
                    old_inst,
                    builder,
                    value_map,
                    false, // check_f32 = false for no-op
                    self.stack_slot_map.as_ref(),
                )?;
            }
        }

        Ok(None) // Instruction was handled
    }
}

/// Copy all instructions from old function to new function.
///
/// Uses the provided InstructionTransformer to handle each instruction.
/// Instructions are processed block-by-block in layout order.
pub fn copy_instructions(
    old_func: &Function,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    block_map: &HashMap<Block, Block>,
    transformer: &mut dyn InstructionTransformer,
) -> Result<(), GlslError> {
    use crate::util::clif_copy::core::ensure_block_params;
    
    // First pass: ensure all block parameters exist before sealing any blocks
    // This ensures that all blocks have the correct number of parameters based on the old function
    let old_blocks: Vec<Block> = old_func.layout.blocks().collect();
    for old_block in &old_blocks {
        let new_block = block_map[old_block];
        // Switch to the block before ensuring parameters (required for append_block_param)
        builder.switch_to_block(new_block);
        // Ensure this block has all the parameters it needs based on the old function
        ensure_block_params(
            old_func,
            *old_block,
            new_block,
            builder,
            value_map,
            |ty| ty, // No-op: preserve types exactly
        )?;
    }

    // Collect instructions for each block
    let mut block_insts: Vec<(Block, Vec<Inst>)> = Vec::new();
    for old_block in &old_blocks {
        let insts: Vec<Inst> = old_func.layout.block_insts(*old_block).collect();
        block_insts.push((*old_block, insts));
    }

    // Second pass: process instructions block-by-block
    // All block parameters have been ensured in the first pass
    // Keep all blocks unsealed while copying instructions so jumps can declare predecessors
    for (old_block, insts) in block_insts {
        let new_block = block_map[&old_block];
        builder.switch_to_block(new_block);

        // Copy instructions (blocks remain unsealed so jumps can declare target blocks as predecessors)
        for old_inst in insts {
            transformer.transform_instruction(
                old_func,
                old_inst,
                builder,
                value_map,
                block_map,
            )?;
        }
    }

    // Seal all blocks after all instructions are copied and all predecessors are declared
    builder.seal_all_blocks();

    Ok(())
}

