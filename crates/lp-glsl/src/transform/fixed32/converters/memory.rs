//! Memory operation conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value, types};
use cranelift_frontend::FunctionBuilder;

use super::helpers::map_value;

/// Convert Load instruction (if it loads F32).
pub(crate) fn convert_load(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &hashbrown::HashMap<cranelift_codegen::ir::Block, cranelift_codegen::ir::Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    // Check if this is an F32 load
    let old_result = old_func.dfg.first_result(old_inst);
    let old_result_type = old_func.dfg.value_type(old_result);

    if old_result_type != types::F32 {
        // Not an F32 load, copy as-is (will be handled by copy_instruction_as_is)
        return Ok(());
    }

    if let InstructionData::Load {
        opcode: _,
        flags,
        arg,
        offset,
    } = inst_data
    {
        // Map address
        let address = map_value(value_map, *arg);
        let target_type = format.cranelift_type();

        // Emit load with new type
        let new_result = builder.ins().load(target_type, *flags, address, *offset);

        // Map result
        value_map.insert(old_result, new_result);
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Load instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert Store instruction (if it stores F32).
pub(crate) fn convert_store(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    block_map: &hashbrown::HashMap<cranelift_codegen::ir::Block, cranelift_codegen::ir::Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Store {
        opcode: _,
        flags,
        args,
        offset,
    } = inst_data
    {
        // Store has args: [value, address]
        let value = args[0];
        let address = args[1];

        // Check if value being stored is F32
        let value_type = old_func.dfg.value_type(value);

        if value_type == types::F32 {
            // Map value (should already be converted)
            let mapped_value = map_value(value_map, value);
            let target_type = format.cranelift_type();

            // Verify mapped value has correct type
            let mapped_type = builder.func.dfg.value_type(mapped_value);
            if mapped_type != target_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Store value type mismatch: expected {:?}, got {:?}",
                        target_type, mapped_type
                    ),
                ));
            }

            // Map address
            let mapped_address = map_value(value_map, address);

            // Emit store with new type
            builder
                .ins()
                .store(*flags, mapped_value, mapped_address, *offset);
        } else {
            // Not an F32 store, copy as-is (will be handled by copy_instruction_as_is)
            // But we still need to map the address in case it's F32-derived
            let mapped_address = map_value(value_map, address);
            let mapped_value = map_value(value_map, value);
            builder
                .ins()
                .store(*flags, mapped_value, mapped_address, *offset);
        }
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Store instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}
