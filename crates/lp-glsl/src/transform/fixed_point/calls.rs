//! Function call converters.

use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as ValueMap, format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::HashMap as ValueMap, format, string::String, vec::Vec};

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{FuncRef, Function, Inst, InstBuilder, InstructionData, Value, types};

use super::transform::WalkCommand;
use crate::transform::fixed_point_math::{
    generate_cos_fixed, generate_sin_fixed, generate_tanh_fixed,
};

/// Convert Call instruction: detect math functions and replace with CORDIC
pub(super) fn convert_call(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    // Extract data before creating cursor to avoid borrow conflicts
    let func_ref = if let InstructionData::Call { func_ref, .. } = &func.dfg.insts[inst] {
        *func_ref
    } else {
        return WalkCommand::Continue;
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
        return WalkCommand::Continue; // No F32 types, skip conversion
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
    let is_tanh = func_name_opt.map_or(false, |name| name == b"tanhf");

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
            // If they're still F32, skip conversion (will be caught by verification)
            if arg_type == types::F32 {
                return WalkCommand::Continue;
            }

            // Use the argument as-is if it's already the target type
            let fixed_arg = if arg_type == target_type {
                arg_val
            } else {
                // Argument is some other type - skip conversion (will be caught by verification)
                return WalkCommand::Continue;
            };

            // Try to generate the appropriate fixed-point function
            let result_opt = if is_sin {
                generate_sin_fixed(&mut cursor, fixed_arg, format).ok()
            } else if is_cos {
                generate_cos_fixed(&mut cursor, fixed_arg, format).ok()
            } else if is_tanh {
                generate_tanh_fixed(&mut cursor, fixed_arg, format).ok()
            } else if is_tan {
                // tan(x) = sin(x) / cos(x)
                // Generate sin and cos, then divide
                let sin_val = match generate_sin_fixed(&mut cursor, fixed_arg, format) {
                    Ok(v) => v,
                    Err(_) => return WalkCommand::Continue, // Will be caught by verification
                };
                let cos_val = match generate_cos_fixed(&mut cursor, fixed_arg, format) {
                    Ok(v) => v,
                    Err(_) => return WalkCommand::Continue, // Will be caught by verification
                };

                // Fixed-point division: result = (sin << shift_amount) / cos
                let shift_amount = format.shift_amount();
                let result = match format {
                    FixedPointFormat::Fixed16x16 => {
                        let sin_ext = cursor
                            .ins()
                            .sextend(cranelift_codegen::ir::types::I64, sin_val);
                        let shift_const = cursor
                            .ins()
                            .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                        let sin_shifted = cursor.ins().ishl(sin_ext, shift_const);
                        let cos_ext = cursor
                            .ins()
                            .sextend(cranelift_codegen::ir::types::I64, cos_val);
                        let div_result = cursor.ins().sdiv(sin_shifted, cos_ext);
                        cursor
                            .ins()
                            .ireduce(cranelift_codegen::ir::types::I32, div_result)
                    }
                    FixedPointFormat::Fixed32x32 => {
                        let sin_ext = cursor
                            .ins()
                            .sextend(cranelift_codegen::ir::types::I128, sin_val);
                        let shift_const = cursor
                            .ins()
                            .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                        let sin_shifted = cursor.ins().ishl(sin_ext, shift_const);
                        let cos_ext = cursor
                            .ins()
                            .sextend(cranelift_codegen::ir::types::I128, cos_val);
                        let div_result = cursor.ins().sdiv(sin_shifted, cos_ext);
                        cursor
                            .ins()
                            .ireduce(cranelift_codegen::ir::types::I64, div_result)
                    }
                };

                Some(result)
            } else {
                // Unknown function - skip conversion (will be caught by verification)
                return WalkCommand::Continue;
            };

            if let Some(result) = result_opt {
                // Successfully generated fixed-point function - use it
                if !old_results.is_empty() {
                    let old_result = old_results[0];

                    // Detach old instruction results
                    cursor.func.dfg.detach_inst_results(inst);

                    // Add to value_map immediately - this maps old F32 result to new I32/I64 result
                    // All uses of old_result will be redirected to result via value_map during forward_walk
                    // Note: We do NOT use change_to_alias here because it's designed for same-type aliasing,
                    // and we're converting from F32 to I32/I64. The value_map mechanism handles cross-type
                    // value replacement correctly.
                    value_map.insert(old_result, result);

                    // Replace old instruction with a harmless instruction (iconst 0)
                    // The alias ensures correctness, this is just to clean up the instruction
                    let target_type = cursor.func.dfg.value_type(result);
                    cursor.func.dfg.replace(inst).iconst(target_type, 0);
                }
                return WalkCommand::Continue;
            }
        }

        // If we get here, the call couldn't be converted to fixed-point math
        // But if it returns F32, we need to convert the result
        // Convert external function signature and handle F32 return values
        if !old_results.is_empty()
            && return_types
                .iter()
                .any(|&t| t == cranelift_codegen::ir::types::F32)
        {
            // Convert the external function signature
            let sig_ref = func.dfg.ext_funcs[func_ref].signature;
            let target_type = format.cranelift_type();

            // Convert parameter types in signature
            for param in &mut func.dfg.signatures[sig_ref].params {
                if param.value_type == types::F32 {
                    param.value_type = target_type;
                }
            }

            // Convert return types in signature
            for ret in &mut func.dfg.signatures[sig_ref].returns {
                if ret.value_type == types::F32 {
                    ret.value_type = target_type;
                }
            }

            // Note: We've modified the signature, but if it's shared with other calls,
            // the modification might not be reflected. We'll handle result type mismatches below.

            // Map call arguments through value_map
            let args: Vec<Value> = func.dfg.inst_args(inst).iter().copied().collect();
            let mapped_args: Vec<Value> = args
                .iter()
                .map(|&arg| *value_map.get(&arg).unwrap_or(&arg))
                .collect();

            // Update call arguments if they changed
            if mapped_args != args {
                for (idx, &mapped_arg) in mapped_args.iter().enumerate() {
                    if mapped_arg != args[idx] {
                        func.dfg.inst_args_mut(inst)[idx] = mapped_arg;
                    }
                }
            }

            // Handle F32 return values - convert them to fixed-point
            // After signature modification, the call should return target_type
            // We need to replace the call instruction so it uses the updated signature
            let needs_conversion = old_results
                .iter()
                .enumerate()
                .any(|(idx, _)| idx < return_types.len() && return_types[idx] == types::F32);

            if needs_conversion {
                // Verify signature was modified correctly
                let modified_sig = &func.dfg.signatures[sig_ref];
                debug_assert!(
                    !modified_sig
                        .returns
                        .iter()
                        .any(|r| r.value_type == types::F32),
                    "Signature should have no F32 returns after modification"
                );

                // Detach old results and replace the call
                // The signature has been modified, so the new call will return target_type
                func.dfg.detach_inst_results(inst);
                func.dfg.replace(inst).call(func_ref, &mapped_args);

                // Verify the new result type matches target_type
                // If it doesn't, the signature modification didn't work as expected
                for (idx, &old_result) in old_results.iter().enumerate() {
                    if idx < func.dfg.inst_results(inst).len() {
                        let new_result = func.dfg.inst_results(inst)[idx];
                        let result_type = func.dfg.value_type(new_result);

                        // If result type is still F32, signature modification didn't work
                        // This can happen if the signature is shared or there's a timing issue
                        // In this case, we need to replace the result value type directly
                        if result_type == types::F32 {
                            // Signature modification didn't work - replace the result value type
                            // This ensures the instruction result itself has the correct type
                            // First detach the result, then replace it with correct type
                            func.dfg.detach_inst_results(inst);
                            let corrected_result = func.dfg.replace_result(new_result, target_type);

                            // Re-attach the corrected result to the instruction
                            // The instruction now has the correct result type
                            func.dfg.make_inst_results(inst, target_type);

                            // Map old result to corrected result
                            value_map.insert(old_result, corrected_result);
                        } else {
                            // Result type is correct - just map it
                            value_map.insert(old_result, new_result);
                        }
                    }
                }

                return WalkCommand::Continue;
            }
        }
    }

    WalkCommand::Continue
}
