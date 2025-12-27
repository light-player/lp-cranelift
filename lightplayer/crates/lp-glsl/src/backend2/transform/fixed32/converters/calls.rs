//! Function call conversion functions.

use crate::backend2::transform::fixed32::converters::map_value;
use crate::backend2::transform::fixed32::signature::convert_signature;
use crate::backend2::transform::fixed32::types::FixedPointFormat;
use crate::error::{ErrorCode, GlslError};
use alloc::{string::String, vec::Vec};
use cranelift_codegen::ir::{
    ExtFuncData, ExternalName, FuncRef, Function, Inst, InstBuilder, InstructionData, SigRef, Value,
};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::FuncId;
use hashbrown::HashMap;

/// Maps an external function reference to a new function reference with converted signature.
///
/// Preserves function identity by name, not just signature. Functions with identical
/// signatures after transformation (e.g., mul_mat2 and mul_vec4) must map to distinct FuncRefs.
pub(crate) fn map_external_function(
    old_func: &Function,
    old_func_ref: FuncRef,
    builder: &mut FunctionBuilder,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
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
                        alloc::format!(
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
    value_map: &mut HashMap<Value, Value>,
    ext_func_map: &mut HashMap<FuncRef, FuncRef>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
    func_id_map: &HashMap<String, FuncId>,
) -> Result<(), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];

    if let InstructionData::Call {
        opcode: _,
        func_ref,
        args,
    } = inst_data
    {
        // Check if this is a TestCase (colocated) function
        let old_ext_func = &old_func.dfg.ext_funcs[*func_ref];
        let new_func_ref = if let ExternalName::TestCase(testcase) = &old_ext_func.name {
            // Extract function name from testcase (remove leading % if present)
            let name_bytes = testcase.raw();
            let name_str = core::str::from_utf8(name_bytes)
                .map_err(|_| {
                    GlslError::new(
                        ErrorCode::E0301,
                        alloc::format!("Invalid testcase name encoding"),
                    )
                })?;
            let func_name = name_str.strip_prefix('%').unwrap_or(name_str);
            
            // Look up FuncId in func_id_map
            let func_id = func_id_map.get(func_name).copied().ok_or_else(|| {
                GlslError::new(
                    ErrorCode::E0301,
                    alloc::format!(
                        "Function '{}' not found in func_id_map (colocated function)",
                        func_name
                    ),
                )
            })?;
            
            // Get the old signature and transform it
            let old_sig_ref = old_ext_func.signature;
            let old_sig = &old_func.dfg.signatures[old_sig_ref];
            let new_sig = convert_signature(old_sig, format);
            
            // Import signature into current function's context
            let new_sig_ref = if let Some(&mapped_sig_ref) = sig_map.get(&old_sig_ref) {
                mapped_sig_ref
            } else {
                let imported_sig_ref = builder.func.import_signature(new_sig);
                sig_map.insert(old_sig_ref, imported_sig_ref);
                imported_sig_ref
            };
            
            // Convert FuncId to UserExternalName (like declare_func_in_func does)
            // This is the correct way to reference colocated functions - TestCase names
            // are only for test CLIF files, but at runtime we use UserExternalName with FuncId
            let user_name_ref = builder.func.declare_imported_user_function(
                cranelift_codegen::ir::UserExternalName {
                    namespace: 0,
                    index: func_id.as_u32(),
                }
            );
            
            let new_ext_func = ExtFuncData {
                name: cranelift_codegen::ir::ExternalName::User(user_name_ref), // Use User, not TestCase!
                signature: new_sig_ref,
                colocated: true,
            };
            
            builder.func.import_function(new_ext_func)
        } else {
            // Use existing logic for external functions
            map_external_function(old_func, *func_ref, builder, ext_func_map, sig_map, format)?
        };

        let old_args = args.as_slice(&old_func.dfg.value_lists);
        let new_args: Vec<Value> = old_args.iter().map(|&v| map_value(value_map, v)).collect();

        let call_inst = builder.ins().call(new_func_ref, &new_args);

        let old_results: Vec<Value> = old_func.dfg.inst_results(old_inst).to_vec();
        let new_results: Vec<Value> = builder.inst_results(call_inst).to_vec();

        if old_results.len() != new_results.len() {
            return Err(GlslError::new(
                ErrorCode::E0301,
                alloc::format!(
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
            alloc::format!("Call instruction has unexpected format: {:?}", inst_data),
        ));
    }

    Ok(())
}

/// Convert CallIndirect instruction.
pub(crate) fn convert_call_indirect(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    sig_map: &mut HashMap<SigRef, SigRef>,
    format: FixedPointFormat,
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
                alloc::format!(
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
            alloc::format!(
                "CallIndirect instruction has unexpected format: {:?}",
                inst_data
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::backend2::transform::fixed32::fixed32_test_util;

    /// Test function calls: add(f32, f32) -> f32
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_call() {
        let clif = r#"
function %add(f32, f32) -> f32 system_v {
block0(v0: f32, v1: f32):
    v2 = fadd v0, v1
    return v2
}

function %main() -> f32 system_v {
    sig0 = (f32, f32) -> f32 system_v
    fn0 = colocated %add sig0

block0:
    v0 = f32const 0x1.8p0
    v1 = f32const 0x1.4p1
    v2 = call fn0(v0, v1)
    return v2
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 4.0);
    }
}

