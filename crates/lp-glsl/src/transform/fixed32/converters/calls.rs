//! Function call conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{
    ExtFuncData, FuncRef, Function, Inst, InstBuilder, InstructionData, SigRef, Value,
};
use cranelift_frontend::FunctionBuilder;

use super::super::signature::convert_signature;
use crate::ir_utils::value_map::map_value;

/// Map external function reference (convert signature and create new function reference).
pub(crate) fn map_external_function(
    old_func: &Function,
    old_func_ref: FuncRef,
    builder: &mut FunctionBuilder,
    ext_func_map: &mut hashbrown::HashMap<FuncRef, FuncRef>,
    sig_map: &mut hashbrown::HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
) -> Result<FuncRef, GlslError> {
    // Check if already mapped
    if let Some(&new_func_ref) = ext_func_map.get(&old_func_ref) {
        return Ok(new_func_ref);
    }

    // Get old external function
    let old_ext_func = &old_func.dfg.ext_funcs[old_func_ref];
    let old_sig_ref = old_ext_func.signature;

    // Convert signature (check if already mapped)
    let new_sig_ref = if let Some(&mapped_sig_ref) = sig_map.get(&old_sig_ref) {
        mapped_sig_ref
    } else {
        let old_sig = &old_func.dfg.signatures[old_sig_ref];
        let new_sig = convert_signature(old_sig, format);
        let new_sig_ref = builder.func.import_signature(new_sig);
        sig_map.insert(old_sig_ref, new_sig_ref);
        new_sig_ref
    };

    // Create new external function
    let new_ext_func = ExtFuncData {
        name: old_ext_func.name.clone(),
        signature: new_sig_ref,
        colocated: old_ext_func.colocated,
    };

    // Import into new function
    let new_func_ref = builder.func.import_function(new_ext_func);

    // Map
    ext_func_map.insert(old_func_ref, new_func_ref);

    Ok(new_func_ref)
}

/// Convert Call instruction.
pub(crate) fn convert_call(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    ext_func_map: &mut hashbrown::HashMap<FuncRef, FuncRef>,
    sig_map: &mut hashbrown::HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &hashbrown::HashMap<cranelift_codegen::ir::Block, cranelift_codegen::ir::Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Call {
        opcode: _,
        func_ref,
        args,
    } = inst_data
    {
        // Convert external function call
        let new_func_ref =
            map_external_function(old_func, *func_ref, builder, ext_func_map, sig_map, format)?;

        // Map call arguments
        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

        // Emit call
        let call_inst = builder.ins().call(new_func_ref, &new_args);

        // Map return values
        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

        // Verify counts match
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

        // Map results
        for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
            value_map.insert(*old_result, *new_result);
        }
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!("Call instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert CallIndirect instruction.
pub(crate) fn convert_call_indirect(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    ext_func_map: &mut hashbrown::HashMap<FuncRef, FuncRef>,
    sig_map: &mut hashbrown::HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    block_map: &hashbrown::HashMap<cranelift_codegen::ir::Block, cranelift_codegen::ir::Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::CallIndirect {
        opcode: _,
        sig_ref,
        args,
    } = inst_data
    {
        // Convert signature reference
        let new_sig_ref = if let Some(&mapped_sig_ref) = sig_map.get(sig_ref) {
            mapped_sig_ref
        } else {
            let old_sig = &old_func.dfg.signatures[*sig_ref];
            let new_sig = convert_signature(old_sig, format);
            let new_sig_ref = builder.func.import_signature(new_sig);
            sig_map.insert(*sig_ref, new_sig_ref);
            new_sig_ref
        };

        // Map arguments (first is function address, rest are call args)
        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let func_addr = map_value(value_map, old_args[0]);
        let call_args: Vec<Value> = old_args[1..]
            .iter()
            .map(|&v| map_value(value_map, v))
            .collect();

        // Emit call_indirect
        let call_inst = builder
            .ins()
            .call_indirect(new_sig_ref, func_addr, &call_args);

        // Map return values
        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

        // Verify counts match
        if old_results.len() != new_results.len() {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "CallIndirect return value count mismatch: old={}, new={}",
                    old_results.len(),
                    new_results.len()
                ),
            ));
        }

        // Map results
        for (old_result, new_result) in old_results.iter().zip(new_results.iter()) {
            value_map.insert(*old_result, *new_result);
        }
    } else {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "CallIndirect instruction has unexpected format: {:?}",
                inst_data
            ),
        ));
    }

    Ok(())
}
