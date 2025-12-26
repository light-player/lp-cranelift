//! Function call conversion functions.

use crate::error::{ErrorCode, GlslError};
use crate::backend::transform::fixed32::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{
    ExtFuncData, ExternalName, FuncRef, Function, Inst, InstBuilder, InstructionData, SigRef,
    Value,
};
use cranelift_frontend::FunctionBuilder;

use super::super::signature::convert_signature;
use crate::backend::ir_utils::value_map::map_value;

/// Maps an external function reference to a new function reference with converted signature.
///
/// Preserves function identity by name, not just signature. Functions with identical
/// signatures after transformation (e.g., mul_mat2 and mul_vec4) must map to distinct FuncRefs.
pub(crate) fn map_external_function(
    old_func: &Function,
    old_func_ref: FuncRef,
    builder: &mut FunctionBuilder,
    ext_func_map: &mut hashbrown::HashMap<FuncRef, FuncRef>,
    sig_map: &mut hashbrown::HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
) -> Result<FuncRef, GlslError> {
    if let Some(&new_func_ref) = ext_func_map.get(&old_func_ref) {
        return Ok(new_func_ref);
    }

    let old_ext_func = &old_func.dfg.ext_funcs[old_func_ref];
    let old_sig_ref = old_ext_func.signature;

    let new_sig_ref = if let Some(&mapped_sig_ref) = sig_map.get(&old_sig_ref) {
        mapped_sig_ref
    } else {
        let old_sig = &old_func.dfg.signatures[old_sig_ref];
        let new_sig = convert_signature(old_sig, format);
        let new_sig_ref = builder.func.import_signature(new_sig);
        sig_map.insert(old_sig_ref, new_sig_ref);
        new_sig_ref
    };

    // Extract actual UserExternalName (namespace:index) from old function and create
    // new reference in new function. This ensures mapping by function identity rather
    // than reference index, which is critical when multiple functions share signatures.
    let new_name = match &old_ext_func.name {
        ExternalName::User(old_user_ref) => {
            let user_name = old_func
                .params
                .user_named_funcs()
                .get(*old_user_ref)
                .cloned()
                .ok_or_else(|| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "UserExternalNameRef {} not found in old function's user_named_funcs",
                            old_user_ref
                        ),
                    )
                })?;
            let new_user_ref = builder.func.declare_imported_user_function(user_name);
            ExternalName::User(new_user_ref)
        }
        _ => old_ext_func.name.clone(),
    };

    let new_ext_func = ExtFuncData {
        name: new_name,
        signature: new_sig_ref,
        colocated: old_ext_func.colocated,
    };

    let new_func_ref = builder.func.import_function(new_ext_func);
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
    _block_map: &hashbrown::HashMap<cranelift_codegen::ir::Block, cranelift_codegen::ir::Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Call {
        opcode: _,
        func_ref,
        args,
    } = inst_data
    {
        let new_func_ref =
            map_external_function(old_func, *func_ref, builder, ext_func_map, sig_map, format)?;

        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

        let call_inst = builder.ins().call(new_func_ref, &new_args);

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
    _ext_func_map: &mut hashbrown::HashMap<FuncRef, FuncRef>,
    sig_map: &mut hashbrown::HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<cranelift_codegen::ir::Block, cranelift_codegen::ir::Block>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::CallIndirect {
        opcode: _,
        sig_ref,
        args,
    } = inst_data
    {
        let new_sig_ref = if let Some(&mapped_sig_ref) = sig_map.get(sig_ref) {
            mapped_sig_ref
        } else {
            let old_sig = &old_func.dfg.signatures[*sig_ref];
            let new_sig = convert_signature(old_sig, format);
            let new_sig_ref = builder.func.import_signature(new_sig);
            sig_map.insert(*sig_ref, new_sig_ref);
            new_sig_ref
        };

        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let func_addr = map_value(value_map, old_args[0]);
        let call_args: Vec<Value> = old_args[1..]
            .iter()
            .map(|&v| map_value(value_map, v))
            .collect();

        let call_inst = builder
            .ins()
            .call_indirect(new_sig_ref, func_addr, &call_args);

        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

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
