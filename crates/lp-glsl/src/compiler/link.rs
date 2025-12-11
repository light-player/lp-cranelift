//! Function linking: rebuild functions with remapped FuncRefs for a new module
//!
//! This module handles the process of taking functions from ClifModule (which were
//! compiled with FuncRefs pointing to a temporary module) and rebuilding them for
//! a new module (JITModule, ObjectModule, etc.) with FuncRefs pointing to the
//! new module's FuncIds.
//!
//! This is similar to the fixed-point transformation in `transform/fixed32/rewrite.rs`,
//! but simpler - we just remap FuncRefs without changing types.
#![allow(dead_code)]

use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{Block, Function, Inst, InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Module};
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Rebuild a function for a new module, remapping FuncRefs to point to new FuncIds
///
/// This creates a new function with the same signature and copies all instructions,
/// remapping FuncRefs in call instructions to point to the new module's FuncIds.
pub fn rebuild_function_for_module<M: Module>(
    old_func: &Function,
    module: &mut M,
    func_id_to_name: &HashMap<u32, String>,
    name_to_id: &HashMap<String, FuncId>,
    new_func_id: FuncId,
) -> Result<Function, GlslError> {
    use cranelift_codegen::ir::{ExternalName, FuncRef, UserFuncName};

    // 1. Create new function with same signature
    let mut new_func = Function::with_name_signature(
        UserFuncName::user(0, new_func_id.as_u32()),
        old_func.signature.clone(),
    );

    // 2. Build mapping from old FuncRef to new FuncRef BEFORE creating builder
    // (to avoid borrow conflicts with new_func)
    let mut func_ref_to_func_id: Vec<(FuncRef, u32)> = Vec::new();
    for (old_func_ref, old_ext_func) in old_func.dfg.ext_funcs.iter() {
        if let ExternalName::User(user_name_ref) = old_ext_func.name {
            // Extract old FuncId from user_named_funcs
            let user_named_funcs = old_func.params.user_named_funcs();
            let old_func_id = if let Some(user_name) = user_named_funcs.get(user_name_ref) {
                user_name.index
            } else {
                // user_named_funcs is empty - match by signature
                let old_sig = &old_func.dfg.signatures[old_ext_func.signature];
                let mut found = false;
                let mut matched_func_id = None;

                // Try to match by comparing signatures
                // We need to find which function in func_id_to_name has a matching signature
                for (func_id_val, func_name) in func_id_to_name.iter() {
                    if let Some(new_func_id) = name_to_id.get(func_name) {
                        let decl = module.declarations().get_function_decl(*new_func_id);
                        // Compare signatures - they should match exactly
                        if decl.signature.params.len() == old_sig.params.len()
                            && decl.signature.returns.len() == old_sig.returns.len()
                        {
                            let params_match =
                                decl.signature.params.iter().zip(old_sig.params.iter()).all(
                                    |(new_param, old_param)| {
                                        new_param.value_type == old_param.value_type
                                            && new_param.purpose == old_param.purpose
                                    },
                                );
                            let returns_match = decl
                                .signature
                                .returns
                                .iter()
                                .zip(old_sig.returns.iter())
                                .all(|(new_ret, old_ret)| {
                                    new_ret.value_type == old_ret.value_type
                                        && new_ret.purpose == old_ret.purpose
                                });

                            if params_match && returns_match {
                                matched_func_id = Some(*func_id_val);
                                found = true;
                                break;
                            }
                        }
                    }
                }

                if !found {
                    // Provide more detailed error message with available signatures
                    let available_sigs: Vec<String> = func_id_to_name
                        .iter()
                        .filter_map(|(func_id_val, func_name)| {
                            name_to_id.get(func_name).map(|new_func_id| {
                                let decl = module.declarations().get_function_decl(*new_func_id);
                                format!(
                                    "  {} (FuncId {}): {:?}",
                                    func_name, func_id_val, decl.signature
                                )
                            })
                        })
                        .collect();

                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "Could not match FuncRef to FuncId - signature matching failed.\n\
                            Looking for signature: {:?}\n\
                            Available signatures:\n{}",
                            old_sig,
                            available_sigs.join("\n")
                        ),
                    ));
                }
                matched_func_id.unwrap()
            };
            func_ref_to_func_id.push((old_func_ref, old_func_id));
        }
    }

    // 3. Create FuncRefs in new module context (before creating builder to avoid borrow conflicts)
    let mut func_ref_map: HashMap<FuncRef, FuncRef> = HashMap::new();
    for (old_func_ref, old_func_id) in &func_ref_to_func_id {
        // Look up function name and get new FuncId
        let callee_name = func_id_to_name.get(old_func_id).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!(
                    "Could not find function name for old FuncId {}",
                    old_func_id
                ),
            )
        })?;
        let new_callee_func_id = name_to_id.get(callee_name).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("Could not find new FuncId for function '{}'", callee_name),
            )
        })?;

        // Create new FuncRef in new module context
        let new_func_ref = module.declare_func_in_func(*new_callee_func_id, &mut new_func);
        func_ref_map.insert(*old_func_ref, new_func_ref);
    }

    // 4. Create builder context (now we can borrow new_func)
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut new_func, &mut builder_ctx);

    // 5. Create blocks and map them (builder now owns new_func)
    let mut block_map: HashMap<Block, Block> = HashMap::new();
    let old_blocks: Vec<Block> = old_func.layout.blocks().collect();
    for old_block in &old_blocks {
        let new_block = builder.create_block();
        block_map.insert(*old_block, new_block);
    }

    // 6. Map function parameters (entry block params)
    let entry_block = old_func
        .layout
        .entry_block()
        .ok_or_else(|| GlslError::new(ErrorCode::E0301, "Function has no entry block"))?;
    let new_entry_block = block_map[&entry_block];

    // Append block parameters for entry block (these are the function parameters)
    // Use the helper method that matches the function signature
    builder.append_block_params_for_function_params(new_entry_block);

    // 7. Map values (parameters)
    let mut value_map: HashMap<Value, Value> = HashMap::new();
    let old_params = old_func.dfg.block_params(entry_block);
    let new_params = builder.block_params(new_entry_block);

    if old_params.len() != new_params.len() {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Entry block parameter count mismatch: old={}, new={}",
                old_params.len(),
                new_params.len()
            ),
        ));
    }

    for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
        value_map.insert(*old_param, *new_param);
    }

    // 8. Copy all instructions block-by-block
    // We need to handle block parameters on-demand as we encounter jumps/brifs
    builder.switch_to_block(new_entry_block);
    builder.seal_block(new_entry_block);

    // Collect blocks and instructions first to avoid borrow conflicts
    let mut block_insts: Vec<(Block, Vec<Inst>)> = Vec::new();
    for old_block in &old_blocks {
        let insts: Vec<Inst> = old_func.layout.block_insts(*old_block).collect();
        block_insts.push((*old_block, insts));
    }

    // Process instructions
    for (old_block, insts) in block_insts {
        let new_block = block_map[&old_block];
        builder.switch_to_block(new_block);

        // Ensure block parameters exist for non-entry blocks
        if old_block != entry_block {
            let old_params = old_func.dfg.block_params(old_block);
            let current_param_count = builder.func.dfg.num_block_params(new_block);
            if old_params.len() > current_param_count {
                for &old_param in old_params.iter().skip(current_param_count) {
                    let param_type = old_func.dfg.value_type(old_param);
                    builder.append_block_param(new_block, param_type);
                }
                // Map newly added parameters
                let new_params = builder.block_params(new_block);
                for i in current_param_count..old_params.len() {
                    if i < old_params.len() && i < new_params.len() {
                        value_map.insert(old_params[i], new_params[i]);
                    }
                }
            }
        }

        // Seal block before processing instructions (FunctionBuilder requirement)
        builder.seal_block(new_block);

        // Copy instructions
        for old_inst in insts {
            copy_instruction(
                old_func,
                old_inst,
                &mut builder,
                &mut value_map,
                &func_ref_map,
                &block_map,
            )?;
        }
    }

    // 9. Seal all blocks
    builder.seal_all_blocks();

    // 10. Finalize builder
    builder.finalize();

    Ok(new_func)
}

/// Copy a single instruction, remapping FuncRefs and Values
fn copy_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    func_ref_map: &HashMap<cranelift_codegen::ir::FuncRef, cranelift_codegen::ir::FuncRef>,
    block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::Opcode;

    let inst_data = &old_func.dfg.insts[old_inst];
    let opcode = inst_data.opcode();

    // Route to appropriate handler
    match opcode {
        Opcode::Call => {
            copy_call_instruction(old_func, old_inst, builder, value_map, func_ref_map)?;
        }
        Opcode::Jump => {
            copy_jump_instruction(old_func, old_inst, builder, value_map, block_map)?;
        }
        Opcode::Brif => {
            copy_brif_instruction(old_func, old_inst, builder, value_map, block_map)?;
        }
        Opcode::Return => {
            copy_return_instruction(old_func, old_inst, builder, value_map)?;
        }
        Opcode::Select => {
            copy_select_instruction(old_func, old_inst, builder, value_map)?;
        }
        _ => {
            // For other instructions, copy as-is (no type conversion needed)
            copy_instruction_as_is(old_func, old_inst, builder, value_map)?;
        }
    }

    Ok(())
}

/// Copy a call instruction, remapping the FuncRef
fn copy_call_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    func_ref_map: &HashMap<cranelift_codegen::ir::FuncRef, cranelift_codegen::ir::FuncRef>,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::InstructionData;

    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Call { func_ref, args, .. } = inst_data {
        // Remap FuncRef
        let new_func_ref = func_ref_map.get(func_ref).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("Could not find new FuncRef for old FuncRef in call instruction"),
            )
        })?;

        // Map arguments
        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

        // Emit call
        let call_inst = builder.ins().call(*new_func_ref, &new_args);

        // Map return values
        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

        if old_results.len() != new_results.len() {
            return Err(GlslError::new(
                ErrorCode::E0301,
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
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Expected Call instruction, got: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Copy a jump instruction, remapping the destination block
fn copy_jump_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::{BlockArg, InstructionData};

    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Jump { destination, .. } = inst_data {
        // Extract destination block and args
        let old_dest_block = destination.block(&old_func.dfg.value_lists);
        let new_dest_block = block_map[&old_dest_block];

        let old_args: Vec<Value> = destination
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();

        // Ensure destination block has the required parameters
        ensure_block_params(old_func, old_dest_block, new_dest_block, builder, value_map)?;

        // Map arguments
        let new_args: Vec<BlockArg> = old_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();

        builder.ins().jump(new_dest_block, &new_args);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Expected Jump instruction, got: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Copy a brif instruction, remapping the destination blocks
fn copy_brif_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::{BlockArg, InstructionData};

    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Brif {
        arg,
        blocks: [block_then_call, block_else_call],
        ..
    } = inst_data
    {
        // Map condition
        let new_cond = map_value(value_map, *arg);

        // Extract blocks from BlockCalls
        let old_then_block = block_then_call.block(&old_func.dfg.value_lists);
        let old_else_block = block_else_call.block(&old_func.dfg.value_lists);

        // Map destination blocks
        let new_then_block = block_map[&old_then_block];
        let new_else_block = block_map[&old_else_block];

        // Map block arguments
        let old_then_args: Vec<Value> = block_then_call
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();
        let old_else_args: Vec<Value> = block_else_call
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();

        // Ensure destination blocks have the required parameters
        ensure_block_params(old_func, old_then_block, new_then_block, builder, value_map)?;
        ensure_block_params(old_func, old_else_block, new_else_block, builder, value_map)?;

        // Map arguments
        let new_then_args: Vec<BlockArg> = old_then_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();
        let new_else_args: Vec<BlockArg> = old_else_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();

        builder.ins().brif(
            new_cond,
            new_then_block,
            &new_then_args,
            new_else_block,
            &new_else_args,
        );
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Expected Brif instruction, got: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Copy a return instruction
fn copy_return_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::InstructionData;

    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::MultiAry { args, .. } = inst_data {
        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();
        builder.ins().return_(&new_args);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Expected Return instruction (MultiAry), got: {:?}",
                inst_data
            ),
        ));
    }

    Ok(())
}

/// Copy a select instruction
fn copy_select_instruction(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::InstructionData;

    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Ternary { args, .. } = inst_data {
        let condition = map_value(value_map, args[0]);
        let true_val = map_value(value_map, args[1]);
        let false_val = map_value(value_map, args[2]);

        let new_result = builder.ins().select(condition, true_val, false_val);
        let old_result = old_func.dfg.first_result(old_inst);
        value_map.insert(old_result, new_result);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Expected Select instruction (Ternary), got: {:?}",
                inst_data
            ),
        ));
    }

    Ok(())
}

/// Copy an instruction as-is (no type conversion, just value mapping)
/// This handles most arithmetic, memory, and other non-control-flow instructions
/// Uses the shared implementation from transform/fixed32/converters
fn copy_instruction_as_is(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
) -> Result<(), GlslError> {
    // Use shared implementation with F32 checking disabled (we're just linking, not converting)
    use crate::transform::fixed32::converters;
    converters::copy_instruction_as_is(
        old_func, old_inst, builder, value_map,
        false, // check_f32 = false for linking (no type conversion)
    )
}

/// Ensure block parameters exist for a target block
fn ensure_block_params(
    old_func: &Function,
    old_block: Block,
    new_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
) -> Result<(), GlslError> {
    let old_params = old_func.dfg.block_params(old_block);
    let current_param_count = builder.func.dfg.num_block_params(new_block);

    if old_params.len() > current_param_count {
        // Add missing parameters
        for &old_param in old_params.iter().skip(current_param_count) {
            let param_type = old_func.dfg.value_type(old_param);
            builder.append_block_param(new_block, param_type);
        }

        // Map newly added parameters
        let new_params = builder.block_params(new_block);
        for i in current_param_count..old_params.len() {
            if i < old_params.len() && i < new_params.len() {
                value_map.insert(old_params[i], new_params[i]);
            }
        }
    }

    Ok(())
}

/// Map a value from old function to new function
fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map
        .get(&old_value)
        .expect("Value not found in value_map - this indicates a bug in instruction copying")
}
