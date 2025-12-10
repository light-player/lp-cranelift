//! Builder-based rewrite approach for fixed-point conversion.
//!
//! This module implements a complete rewrite of functions using FunctionBuilder,
//! creating a new function from scratch rather than mutating in place.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::HashMap, vec::Vec};

use cranelift_codegen::ir::{
    AbiParam, Block, FuncRef, Function, Inst, InstructionData, Opcode, SigRef, Signature, Value,
    ValueList, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

use super::converters;

/// Context for rewriting a function from F32 to fixed-point.
///
/// This holds all the state needed during the rewrite process:
/// - The old function (read-only reference)
/// - The new function being built
/// - Builder context (needed for FunctionBuilder)
/// - Maps for blocks and values (old → new)
pub struct RewriteContext<'a> {
    /// Read-only reference to the original function
    pub old_func: &'a Function,
    /// New function being built
    pub new_func: Function,
    /// Builder context (needed for FunctionBuilder)
    pub builder_ctx: FunctionBuilderContext,
    /// Map from old blocks to new blocks
    pub block_map: HashMap<Block, Block>,
    /// Map from old values to new values
    pub value_map: HashMap<Value, Value>,
    /// Map from old function references to new function references
    pub ext_func_map: HashMap<FuncRef, FuncRef>,
    /// Map from old signature references to new signature references
    pub sig_map: HashMap<SigRef, SigRef>,
    /// Fixed-point format to use
    pub format: FixedPointFormat,
}

/// Convert function signature: F32 params/returns → I32/I64
pub(super) fn convert_signature(old_sig: &Signature, format: FixedPointFormat) -> Signature {
    let target_type = format.cranelift_type();
    let mut new_sig = Signature::new(old_sig.call_conv);

    // Convert parameters
    for param in &old_sig.params {
        let new_type = if param.value_type == types::F32 {
            target_type
        } else {
            param.value_type
        };
        new_sig.params.push(AbiParam::new(new_type));
    }

    // Convert return types
    for ret in &old_sig.returns {
        let new_type = if ret.value_type == types::F32 {
            target_type
        } else {
            ret.value_type
        };
        new_sig.returns.push(AbiParam::new(new_type));
    }

    new_sig
}

/// Main entry point for rewriting a function.
///
/// Creates a new function with converted signature and converts all
/// instructions from F32 to fixed-point representation.
pub fn rewrite_function(
    old_func: &Function,
    format: FixedPointFormat,
) -> Result<Function, GlslError> {
    // 1. Convert signature
    let new_sig = convert_signature(&old_func.signature, format);

    // 2. Create new function
    let mut new_func = Function::with_name_signature(old_func.name.clone(), new_sig);

    // 3. Create builder context
    let mut builder_ctx = FunctionBuilderContext::new();

    // 4. Create a single builder that we'll reuse throughout
    let mut builder = FunctionBuilder::new(&mut new_func, &mut builder_ctx);

    // 5. Create context
    // Note: new_func is borrowed by builder, and builder_ctx is borrowed by builder
    // So we create dummies for the context. We'll return the real new_func after builder is dropped
    let new_sig_clone = convert_signature(&old_func.signature, format);
    let mut ctx = RewriteContext {
        old_func,
        new_func: Function::with_name_signature(old_func.name.clone(), new_sig_clone), // Dummy
        builder_ctx: FunctionBuilderContext::new(),                                    // Dummy
        block_map: HashMap::new(),
        value_map: HashMap::new(),
        ext_func_map: HashMap::new(),
        sig_map: HashMap::new(),
        format,
    };

    // 6. Build blocks and map parameters
    create_and_map_blocks(&mut ctx, &mut builder)?;

    // 7. Map function parameters (verify entry block params)
    map_function_params(&mut ctx, &mut builder)?;

    // 8. Convert instructions (this will switch to blocks as needed)
    convert_all_instructions(&mut ctx, &mut builder)?;

    // 9. Seal all blocks now that all instructions are converted
    builder.seal_all_blocks();

    // 10. Finalize builder (this clears the builder context)
    builder.finalize();

    // 11. Return new function (builder is dropped, so we can return new_func)
    // Note: builder.func points to new_func, so they're the same
    Ok(new_func)
}

/// Create all blocks in the new function and map block parameters.
fn create_and_map_blocks(
    ctx: &mut RewriteContext,
    builder: &mut FunctionBuilder,
) -> Result<(), GlslError> {
    // Collect blocks first to avoid borrow conflicts
    let old_blocks: Vec<Block> = ctx.old_func.layout.blocks().collect();
    let old_entry_block = ctx.old_func.layout.entry_block();

    // Create blocks in same order as original
    for old_block in &old_blocks {
        let new_block = builder.create_block();
        ctx.block_map.insert(*old_block, new_block);
    }

    // Handle entry block specially - use function parameters
    if let Some(old_entry) = old_entry_block {
        let new_entry = ctx.block_map[&old_entry];

        // For entry block, use function parameters (this creates params matching the signature)
        builder.append_block_params_for_function_params(new_entry);

        // Map old entry block params to new entry block params
        let old_params = ctx.old_func.dfg.block_params(old_entry);
        let new_params = builder.block_params(new_entry);

        // Verify counts match
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

        // Map old params to new params
        for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
            ctx.value_map.insert(*old_param, *new_param);
        }
    }

    // Now map block parameters for non-entry blocks
    for old_block in ctx.old_func.layout.blocks() {
        // Skip entry block - already handled
        if Some(old_block) == old_entry_block {
            continue;
        }

        let new_block = ctx.block_map[&old_block];
        map_block_params(
            ctx.old_func,
            old_block,
            new_block,
            builder,
            &mut ctx.value_map,
            ctx.format,
        )?;
    }

    // Don't seal blocks here - we'll seal them after converting all instructions
    // This allows us to switch between blocks freely during instruction conversion

    Ok(())
}

/// Map block parameters from old block to new block.
fn map_block_params(
    old_func: &Function,
    old_block: Block,
    new_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let old_params = old_func.dfg.block_params(old_block);
    let target_type = format.cranelift_type();

    // Determine new parameter types
    let mut new_param_types = Vec::new();
    for &old_param in old_params {
        let old_type = old_func.dfg.value_type(old_param);
        let new_type = if old_type == types::F32 {
            target_type
        } else {
            old_type
        };
        new_param_types.push(new_type);
    }

    // Declare block parameters in new block
    for &param_type in &new_param_types {
        builder.append_block_param(new_block, param_type);
    }

    // Map old params to new params
    let new_params = builder.block_params(new_block);

    // Verify counts match
    if old_params.len() != new_params.len() {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Block parameter count mismatch: old={}, new={}",
                old_params.len(),
                new_params.len()
            ),
        ));
    }

    // Map values and verify types
    for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
        value_map.insert(*old_param, *new_param);

        // Verify types are correct
        let old_type = old_func.dfg.value_type(*old_param);
        let new_type = builder.func.dfg.value_type(*new_param);

        if old_type == types::F32 {
            if new_type != target_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "F32 block parameter not converted: expected {:?}, got {:?}",
                        target_type, new_type
                    ),
                ));
            }
        } else {
            if new_type != old_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Non-F32 block parameter type changed: old={:?}, new={:?}",
                        old_type, new_type
                    ),
                ));
            }
        }
    }

    Ok(())
}

/// Map function parameters (verify entry block params match function signature).
fn map_function_params(
    ctx: &mut RewriteContext,
    builder: &mut FunctionBuilder,
) -> Result<(), GlslError> {
    // Get entry block
    let entry_block = ctx
        .old_func
        .layout
        .entry_block()
        .ok_or_else(|| GlslError::new(ErrorCode::E0301, "Function has no entry block"))?;

    let new_entry_block = ctx.block_map[&entry_block];

    // Function parameters are the block parameters of the entry block
    // They should already be mapped in create_and_map_blocks
    // But verify they're correct
    let old_params = ctx.old_func.dfg.block_params(entry_block);
    let new_params = builder.block_params(new_entry_block);

    // Verify counts match
    if old_params.len() != new_params.len() {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Function parameter count mismatch: old={}, new={}",
                old_params.len(),
                new_params.len()
            ),
        ));
    }

    // Verify types are correct
    let target_type = ctx.format.cranelift_type();
    for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
        let old_type = ctx.old_func.dfg.value_type(*old_param);
        let new_type = builder.func.dfg.value_type(*new_param);

        if old_type == types::F32 {
            if new_type != target_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "F32 function parameter not converted: expected {:?}, got {:?}",
                        target_type, new_type
                    ),
                ));
            }
        } else {
            if new_type != old_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Non-F32 function parameter type changed: old={:?}, new={:?}",
                        old_type, new_type
                    ),
                ));
            }
        }

        // Verify mapping exists
        if ctx.value_map.get(old_param) != Some(new_param) {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "Function parameter not mapped: old_param={:?}, expected new_param={:?}",
                    old_param, new_param
                ),
            ));
        }
    }

    Ok(())
}

/// Setup entry block (switch to it).
fn setup_entry_block(
    ctx: &mut RewriteContext,
    builder: &mut FunctionBuilder,
) -> Result<(), GlslError> {
    let entry_block = ctx
        .old_func
        .layout
        .entry_block()
        .ok_or_else(|| GlslError::new(ErrorCode::E0301, "Function has no entry block"))?;

    let new_entry_block = ctx.block_map[&entry_block];

    // Switch to entry block (it's already sealed by seal_all_blocks())
    builder.switch_to_block(new_entry_block);

    Ok(())
}

/// Traverse all instructions and convert them.
fn convert_all_instructions(
    ctx: &mut RewriteContext,
    builder: &mut FunctionBuilder,
) -> Result<(), GlslError> {
    // Collect blocks and instructions first to avoid borrow conflicts
    let old_blocks: Vec<Block> = ctx.old_func.layout.blocks().collect();
    let mut block_insts: Vec<(Block, Vec<Inst>)> = Vec::new();
    for old_block in &old_blocks {
        let insts: Vec<Inst> = ctx.old_func.layout.block_insts(*old_block).collect();
        block_insts.push((*old_block, insts));
    }

    // Now process instructions
    // We need to access block_map (immutable) and value_map (mutable) separately
    let block_map = &ctx.block_map;
    for (old_block, insts) in block_insts {
        let new_block = block_map[&old_block];
        builder.switch_to_block(new_block);

        for old_inst in insts {
            convert_instruction(
                ctx.old_func,
                old_inst,
                old_block,
                new_block,
                builder,
                &mut ctx.value_map,
                &mut ctx.ext_func_map,
                &mut ctx.sig_map,
                ctx.format,
                block_map,
            )?;
        }
    }

    Ok(())
}

/// Convert a single instruction.
fn convert_instruction(
    old_func: &Function,
    old_inst: Inst,
    old_block: Block,
    new_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    // Don't switch blocks here - we're already on the correct block from convert_all_instructions
    // builder.switch_to_block(new_block);

    // Copy source location (skip for now - RelSourceLoc conversion will be handled later)
    // if let Some(srcloc) = old_func.srclocs.get(old_inst) {
    //     builder.set_srcloc(*srcloc);
    // }

    let opcode = old_func.dfg.insts[old_inst].opcode();

    // Route to appropriate converter
    match opcode {
        Opcode::F32const => {
            converters::constants::convert_f32const(
                old_func, old_inst, builder, value_map, format,
            )?;
        }
        Opcode::Fadd => {
            converters::arithmetic::convert_fadd(old_func, old_inst, builder, value_map, format)?;
        }
        Opcode::Fsub => {
            converters::arithmetic::convert_fsub(old_func, old_inst, builder, value_map, format)?;
        }
        Opcode::Fmul => {
            converters::arithmetic::convert_fmul(old_func, old_inst, builder, value_map, format)?;
        }
        Opcode::Fdiv => {
            converters::arithmetic::convert_fdiv(old_func, old_inst, builder, value_map, format)?;
        }
        Opcode::Fneg => {
            converters::arithmetic::convert_fneg(old_func, old_inst, builder, value_map, format)?;
        }
        Opcode::Fabs => {
            converters::arithmetic::convert_fabs(old_func, old_inst, builder, value_map, format)?;
        }
        Opcode::Jump => {
            converters::control::convert_jump(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Brif => {
            converters::control::convert_brif(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::BrTable => {
            converters::control::convert_br_table(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Return => {
            converters::control::convert_return(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Select => {
            converters::control::convert_select(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Load => {
            converters::memory::convert_load(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Store => {
            converters::memory::convert_store(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Call => {
            converters::calls::convert_call(
                old_func,
                old_inst,
                builder,
                value_map,
                ext_func_map,
                sig_map,
                format,
                block_map,
            )?;
        }
        Opcode::CallIndirect => {
            converters::calls::convert_call_indirect(
                old_func,
                old_inst,
                builder,
                value_map,
                ext_func_map,
                sig_map,
                format,
                block_map,
            )?;
        }
        Opcode::Fcmp => {
            converters::comparison::convert_fcmp(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Fmax => {
            converters::comparison::convert_fmax(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Fmin => {
            converters::comparison::convert_fmin(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Sqrt => {
            converters::math::convert_sqrt(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Ceil => {
            converters::math::convert_ceil(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::Floor => {
            converters::math::convert_floor(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::FcvtFromSint => {
            converters::conversions::convert_fcvt_from_sint(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        Opcode::FcvtFromUint => {
            converters::conversions::convert_fcvt_from_uint(
                old_func, old_inst, builder, value_map, format, block_map,
            )?;
        }
        _ => {
            // For non-F32 instructions, copy as-is
            copy_instruction_as_is(old_func, old_inst, builder, value_map, format, block_map)?;
        }
    }

    Ok(())
}

/// Map an old value to its new equivalent.
pub(super) fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
}

/// Map multiple values.
pub(super) fn map_values(value_map: &HashMap<Value, Value>, old_values: &[Value]) -> Vec<Value> {
    old_values
        .iter()
        .map(|&v| map_value(value_map, v))
        .collect()
}

/// Copy a non-F32 instruction as-is (for instructions that don't need conversion).
///
/// This handles instructions that don't involve F32 types and can be copied
/// directly, mapping all values through value_map.
///
/// Note: This is a simplified implementation. For complex instructions,
/// explicit converters should be created.
fn copy_instruction_as_is(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    _format: FixedPointFormat,
    _block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    let opcode = old_func.dfg.insts[old_inst].opcode();
    let inst_data = &old_func.dfg.insts[old_inst];

    // First, check if this instruction involves F32 types
    // If it does, it should have been converted already
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

    // If no results, this is typically a terminator or side-effect only instruction
    // These should have been handled by explicit converters, but if they fall through,
    // we can safely skip them (they don't produce values to map)
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
