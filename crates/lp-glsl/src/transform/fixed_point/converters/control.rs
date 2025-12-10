//! Control flow instruction conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{
    Block, BlockArg, BlockCall, Function, Inst, InstBuilder, InstructionData, JumpTable,
    JumpTableData, Value,
};
use cranelift_frontend::FunctionBuilder;

use super::super::rewrite::map_value;

/// Convert Jump instruction.
pub(crate) fn convert_jump(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<Block, Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Jump { destination, .. } = inst_data {
        // Map destination block
        let old_dest_block = destination.block(&old_func.dfg.value_lists);
        let new_dest_block = block_map[&old_dest_block];

        // Map arguments
        let old_args: Vec<Value> = destination
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();
        let new_args: Vec<BlockArg> = old_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();

        // Emit jump
        builder.ins().jump(new_dest_block, &new_args);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Jump instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert Brif instruction.
pub(crate) fn convert_brif(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<Block, Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Brif {
        arg,
        blocks: [block_then_call, block_else_call],
        ..
    } = inst_data
    {
        // Map condition
        let condition = map_value(value_map, *arg);

        // Extract blocks from BlockCalls
        let old_then_block = block_then_call.block(&old_func.dfg.value_lists);
        let old_else_block = block_else_call.block(&old_func.dfg.value_lists);

        // Map destination blocks
        let new_then_block = block_map[&old_then_block];
        let new_else_block = block_map[&old_else_block];

        // Map block arguments (if any)
        let old_then_args: Vec<Value> = block_then_call
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();
        let old_else_args: Vec<Value> = block_else_call
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();

        let new_then_args: Vec<BlockArg> = old_then_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();
        let new_else_args: Vec<BlockArg> = old_else_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();

        // Emit brif
        builder.ins().brif(
            condition,
            new_then_block,
            &new_then_args,
            new_else_block,
            &new_else_args,
        );
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Brif instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert BrTable instruction.
pub(crate) fn convert_br_table(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<Block, Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::BranchTable { arg, table, .. } = inst_data {
        // Map condition
        let condition = map_value(value_map, *arg);

        // Get old jump table
        let old_table = &old_func.dfg.jump_tables[*table];

        // Map default destination (first element in jump table)
        let old_default_block_call = old_table.default_block();
        let old_default_block = old_default_block_call.block(&old_func.dfg.value_lists);
        let new_default_block = block_map[&old_default_block];

        // Map default block arguments
        let old_default_args: Vec<Value> = old_default_block_call
            .args(&old_func.dfg.value_lists)
            .filter_map(|arg| arg.as_value())
            .collect();
        let new_default_args: Vec<BlockArg> = old_default_args
            .iter()
            .map(|&v| map_value(value_map, v).into())
            .collect();
        let new_default_block_call = builder
            .func
            .dfg
            .block_call(new_default_block, &new_default_args);

        // Map table destinations
        let mut new_table_blocks = Vec::new();
        for old_block_call in old_table.as_slice() {
            let old_block = old_block_call.block(&old_func.dfg.value_lists);
            let new_block = block_map[&old_block];

            // Map block arguments
            let old_args: Vec<Value> = old_block_call
                .args(&old_func.dfg.value_lists)
                .filter_map(|arg| arg.as_value())
                .collect();
            let new_args: Vec<BlockArg> = old_args
                .iter()
                .map(|&v| map_value(value_map, v).into())
                .collect();
            let new_block_call = builder.func.dfg.block_call(new_block, &new_args);
            new_table_blocks.push(new_block_call);
        }

        // Create new jump table
        let new_table = builder.create_jump_table(JumpTableData::new(
            new_default_block_call,
            &new_table_blocks,
        ));

        // Emit br_table
        builder.ins().br_table(condition, new_table);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("BrTable instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert Return instruction.
pub(crate) fn convert_return(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<Block, Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::MultiAry {
        opcode: _, args, ..
    } = inst_data
    {
        // Map return arguments
        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

        // Emit return
        builder.ins().return_(&new_args);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Return instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert Select instruction.
pub(crate) fn convert_select(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut std::collections::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &std::collections::HashMap<Block, Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Ternary { args, .. } = inst_data {
        // Map operands
        let condition = map_value(value_map, args[0]);
        let true_val = map_value(value_map, args[1]);
        let false_val = map_value(value_map, args[2]);

        // Emit select
        let new_result = builder.ins().select(condition, true_val, false_val);

        // Map result
        let old_result = old_func.dfg.first_result(old_inst);
        value_map.insert(old_result, new_result);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Select instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

