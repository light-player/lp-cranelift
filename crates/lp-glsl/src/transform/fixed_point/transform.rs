//! Main transformation orchestration and signature conversion.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as ValueMap, format, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::HashMap as ValueMap, format, vec::Vec};

use cranelift_codegen::ir::{Block, Function, Inst, Opcode, Value};

use super::arithmetic::{
    convert_fabs, convert_fadd, convert_fdiv, convert_fmul, convert_fneg, convert_fsub,
};
use super::calls::convert_call;
use super::comparison::{convert_fcmp, convert_fmax, convert_fmin};
use super::constants::convert_f32const;
use super::control::convert_select;
use super::math::{convert_ceil, convert_floor, convert_sqrt};
use super::memory::{convert_load, convert_store};

/// Convert all float operations in a function to fixed-point.
///
/// This pass:
/// 1. Converts function signature (F32 → I32/I64)
/// 2. Walks through all instructions and replaces float ops with fixed-point ops
/// 3. Builds a value replacement map as we go
/// 4. Updates all value uses with the map
/// 5. Verifies the function is still valid
pub fn convert_floats_to_fixed(
    func: &mut Function,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // 1. Convert signature
    convert_signature(func, format);

    // 2. Build a value replacement map (old F32 value -> new I32/I64 value)
    let mut value_map: ValueMap<Value, Value> = ValueMap::new();

    // 3. Walk all blocks and instructions to convert them
    // Collect instructions first to avoid borrow issues
    let mut insts_to_convert: Vec<(Block, Inst)> = Vec::new();
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            insts_to_convert.push((block, inst));
        }
    }

    // 4. Convert each instruction, building the value map
    for (_block, inst) in insts_to_convert {
        convert_instruction(func, inst, format, &mut value_map)?;
    }

    // 5. Apply the value map to all instructions
    // Collect blocks and instructions first to avoid borrow checker issues
    let blocks_and_insts: Vec<(Block, Vec<Inst>)> = func
        .layout
        .blocks()
        .map(|block| {
            let insts = func.layout.block_insts(block).collect();
            (block, insts)
        })
        .collect();

    for (_block, insts) in blocks_and_insts {
        for inst in insts {
            func.dfg
                .map_inst_values(inst, |val| *value_map.get(&val).unwrap_or(&val));
        }
    }

    // 6. Verify function is still valid
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

/// Convert a single instruction
fn convert_instruction(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let opcode = func.dfg.insts[inst].opcode();

    match opcode {
        Opcode::F32const => convert_f32const(func, inst, format, value_map)?,
        Opcode::Fadd => convert_fadd(func, inst, format, value_map)?,
        Opcode::Fsub => convert_fsub(func, inst, format, value_map)?,
        Opcode::Fmul => convert_fmul(func, inst, format, value_map)?,
        Opcode::Fdiv => convert_fdiv(func, inst, format, value_map)?,
        Opcode::Fcmp => convert_fcmp(func, inst, format, value_map)?,
        Opcode::Fneg => convert_fneg(func, inst, format, value_map)?,
        Opcode::Fabs => convert_fabs(func, inst, format, value_map)?,
        Opcode::Fmax => convert_fmax(func, inst, format, value_map)?,
        Opcode::Fmin => convert_fmin(func, inst, format, value_map)?,
        Opcode::Sqrt => convert_sqrt(func, inst, format, value_map)?,
        Opcode::Ceil => convert_ceil(func, inst, format, value_map)?,
        Opcode::Floor => convert_floor(func, inst, format, value_map)?,
        Opcode::Select => convert_select(func, inst, format, value_map)?,
        Opcode::Load => convert_load(func, inst, format, value_map)?,
        Opcode::Store => convert_store(func, inst, format, value_map)?,
        Opcode::Call => convert_call(func, inst, format, value_map)?,
        _ => {
            // Other instructions don't need conversion
        }
    }

    Ok(())
}
