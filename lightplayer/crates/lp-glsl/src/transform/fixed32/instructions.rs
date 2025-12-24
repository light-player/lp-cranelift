//! Instruction conversion routing for fixed-point transformation.

use crate::error::GlslError;
use crate::transform::fixed32::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{Block, FuncRef, Function, Inst, Opcode, SigRef, Value};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

use super::converters;

/// Traverse all instructions and convert them.
pub(super) fn convert_all_instructions(
    old_func: &Function,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    // Collect blocks and instructions first to avoid borrow conflicts
    let old_blocks: Vec<Block> = old_func.layout.blocks().collect();
    let mut block_insts: Vec<(Block, Vec<Inst>)> = Vec::new();
    for old_block in &old_blocks {
        let insts: Vec<Inst> = old_func.layout.block_insts(*old_block).collect();
        block_insts.push((*old_block, insts));
    }

    // Now process instructions
    for (old_block, insts) in block_insts {
        let new_block = block_map[&old_block];
        builder.switch_to_block(new_block);

        for old_inst in insts {
            convert_instruction(
                old_func,
                old_inst,
                old_block,
                new_block,
                builder,
                value_map,
                ext_func_map,
                sig_map,
                format,
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
    _old_block: Block,
    _new_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    // Don't switch blocks here - we're already on the correct block from convert_all_instructions
    // The new_block parameter is kept for API consistency but not used

    // Copy source location from old instruction to new instructions
    // This ensures source locations are preserved through the fixed32 transform
    // Convert RelSourceLoc to absolute SourceLoc
    let srcloc = old_func.srcloc(old_inst);
    if !srcloc.is_default() {
        builder.set_srcloc(srcloc);
    }

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
        Opcode::Sqrt => {
            converters::math::convert_sqrt(
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

/// Copy a non-F32 instruction as-is (for instructions that don't need conversion).
///
/// Delegates to the shared implementation in converters::instruction_copy.
fn copy_instruction_as_is(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    _format: FixedPointFormat,
    _block_map: &HashMap<Block, Block>,
) -> Result<(), GlslError> {
    // Use shared implementation with F32 checking enabled
    converters::copy_instruction_as_is(
        old_func, old_inst, builder, value_map,
        true, // check_f32 = true for fixed-point conversion
    )
}

