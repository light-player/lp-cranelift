//! Function call converters.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as ValueMap, format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::HashMap as ValueMap, format, string::String, vec::Vec};

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{FuncRef, Function, Inst, InstructionData, Value, types};

use crate::transform::fixed_point_math::{generate_cos_fixed, generate_sin_fixed};

/// Convert Call instruction: detect math functions and replace with CORDIC
pub(super) fn convert_call(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    // Extract data before creating cursor to avoid borrow conflicts
    let func_ref = if let InstructionData::Call { func_ref, .. } = &func.dfg.insts[inst] {
        *func_ref
    } else {
        return Ok(());
    };

    // Get the function signature and extract needed info
    let sig_ref = func.dfg.ext_funcs[func_ref].signature;
    let param_types: Vec<types::Type> = func.dfg.signatures[sig_ref]
        .params
        .iter()
        .map(|p| p.value_type)
        .collect();
    let return_types: Vec<types::Type> = func.dfg.signatures[sig_ref]
        .returns
        .iter()
        .map(|r| r.value_type)
        .collect();

    // Check if this call needs conversion (has F32 params or returns)
    let needs_conversion = param_types.iter().any(|&t| t == types::F32)
        || return_types.iter().any(|&t| t == types::F32);

    if !needs_conversion {
        return Ok(()); // No F32 types, skip conversion
    }

    // Collect data before creating cursor to avoid borrow conflicts
    let args: Vec<Value> = func.dfg.inst_args(inst).iter().copied().collect();
    let old_results: Vec<Value> = func.dfg.inst_results(inst).iter().copied().collect();

    // Detect math functions by signature: (f32) -> f32 for sin/cos/tan/etc, (f32, f32) -> f32 for atan2
    let is_math_function = param_types.len() == 1
        && param_types[0] == types::F32
        && return_types.len() == 1
        && return_types[0] == types::F32;

    // Try to identify which math function this is (extract before creating cursor)
    // Extract function name info and clone bytes to avoid borrow conflicts
    let (func_name_bytes, ext_func_name_str) = {
        let ext_func = &func.dfg.ext_funcs[func_ref];
        let func_name_opt: Option<&[u8]> = match &ext_func.name {
            cranelift_codegen::ir::ExternalName::TestCase(name) => {
                // TestCase names contain the function name as bytes
                Some(name.raw())
            }
            cranelift_codegen::ir::ExternalName::User(_) => {
                // For User names, we can't easily get the string name
                // These are created when module supports imports
                None
            }
            _ => None,
        };
        // Clone bytes to break borrow
        let func_name_bytes = func_name_opt.map(|b| b.to_vec());
        let ext_func_name_str = format!("{:?}", ext_func.name);
        (func_name_bytes, ext_func_name_str)
    };

    let func_name_opt: Option<&[u8]> = func_name_bytes.as_deref();

    // Try to match function name
    let is_sin = func_name_opt.map_or(false, |name| name == b"sinf");
    let is_cos = func_name_opt.map_or(false, |name| name == b"cosf");
    let is_tan = func_name_opt.map_or(false, |name| name == b"tanf");

    // Get argument info before creating cursor to avoid borrow conflicts
    let arg_info = if is_math_function && args.len() == 1 {
        let arg_val = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg_type = func.dfg.value_type(arg_val);
        let target_type = format.cranelift_type();
        Some((arg_val, arg_type, target_type))
    } else {
        None
    };

    {
        // Scope for cursor
        let mut cursor = FuncCursor::new(func).at_inst(inst);

        if let Some((arg_val, arg_type, target_type)) = arg_info {
            // Arguments should already be converted to fixed-point by the time we process the call
            // If they're still F32, that's an error - we can't convert F32 to fixed-point without float ops
            if arg_type == types::F32 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "call argument still F32 after conversion (function call must use fixed-point arguments)"
                    ),
                ));
            }

            // Use the argument as-is if it's already the target type
            let fixed_arg = if arg_type == target_type {
                arg_val
            } else {
                // Argument is some other type - shouldn't happen for math functions
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!("call argument has unexpected type after conversion"),
                ));
            };

            // Try to generate the appropriate fixed-point function
            let result_opt = if is_sin {
                generate_sin_fixed(&mut cursor, fixed_arg, format).ok()
            } else if is_cos {
                generate_cos_fixed(&mut cursor, fixed_arg, format).ok()
            } else if is_tan {
                // tan(x) = sin(x) / cos(x)
                // TODO: Implement tan using sin/cos division with fixed-point math
                // For now, error out - we don't support float operations
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "tan() function not yet implemented in fixed-point (needs fixed-point division)"
                    ),
                ));
            } else {
                // Unknown function - error out instead of using float conversion
                let func_name_str = func_name_bytes
                    .as_ref()
                    .map(|n| String::from_utf8_lossy(n).into_owned())
                    .unwrap_or_else(|| ext_func_name_str.clone());
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "function '{}' does not have fixed-point implementation and float operations are not supported",
                        func_name_str
                    ),
                ));
            };

            if let Some(result) = result_opt {
                // Successfully generated fixed-point function - use it
                if !old_results.is_empty() {
                    value_map.insert(old_results[0], result);
                }

                // Remove the old call instruction
                cursor.func.dfg.detach_inst_results(inst);
                cursor.goto_inst(inst);
                cursor.remove_inst();
                return Ok(());
            }
        }

        // If we get here, the call couldn't be converted to fixed-point math
        // Return an error instead of generating float instructions
        let func_name_str = func_name_bytes
            .as_ref()
            .map(|n| String::from_utf8_lossy(n).into_owned())
            .unwrap_or_else(|| ext_func_name_str);
        return Err(GlslError::new(
            ErrorCode::E0400,
            format!(
                "call to '{}' requires float operations which are not supported",
                func_name_str
            ),
        ));
    }
}
