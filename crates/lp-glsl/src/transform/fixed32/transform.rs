//! Main transformation entry point.
//!
//! This module provides the public API for converting functions from F32 to fixed-point.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::Function;

use super::rewrite;

/// Convert all float operations in a function to fixed-point.
///
/// This pass uses a builder-based rewrite approach:
/// 1. Creates a new function with converted signature
/// 2. Traverses the old function and converts instructions
/// 3. Verifies no F32 values remain
/// 4. Verifies the function is still valid
pub fn convert_floats_to_fixed(
    func: &mut Function,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Use rewrite approach
    let new_func = rewrite::rewrite_function(func, format)?;
    *func = new_func;

    // Verify no F32 values remain
    verify_no_f32_values(func)?;

    // Verify function is still valid
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

/// Verify that no F32 values remain in the function after transformation
fn verify_no_f32_values(func: &Function) -> Result<(), GlslError> {
    verify_signature(func)?;
    verify_block_params(func)?;
    verify_instructions(func)?;
    verify_jump_tables(func)?;
    Ok(())
}

/// Verify function signature has no F32 types
fn verify_signature(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    // Check parameters
    for (idx, param) in func.signature.params.iter().enumerate() {
        if param.value_type == types::F32 {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "F32 parameter still present after fixed-point transformation: param[{}] has type F32",
                    idx
                ),
            ));
        }
    }

    // Check return types
    for (idx, ret) in func.signature.returns.iter().enumerate() {
        if ret.value_type == types::F32 {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "F32 return type still present after fixed-point transformation: return[{}] has type F32",
                    idx
                ),
            ));
        }
    }

    Ok(())
}

/// Verify block parameters have no F32 types
fn verify_block_params(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    for block in func.layout.blocks() {
        for (idx, &param) in func.dfg.block_params(block).iter().enumerate() {
            if func.dfg.value_type(param) == types::F32 {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "F32 block parameter still present after fixed-point transformation: block = `{}`, param[{}] = `{}` has type F32",
                        block, idx, param
                    ),
                ));
            }
        }
    }

    Ok(())
}

/// Verify all instructions have no F32 values
fn verify_instructions(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            // Check instruction results
            for &result in func.dfg.inst_results(inst) {
                let resolved_result = func.dfg.resolve_aliases(result);
                if func.dfg.value_type(resolved_result) == types::F32 {
                    return Err(GlslError::new(
                        ErrorCode::E0301,
                        format!(
                            "F32 result still present after fixed-point transformation: block = `{}`, inst = `{}`, result = `{}` (resolved: `{}`)",
                            block,
                            func.dfg.display_inst(inst),
                            result,
                            resolved_result
                        ),
                    ));
                }
            }

            // Check instruction operands
            for (idx, &arg) in func.dfg.inst_args(inst).iter().enumerate() {
                if func.dfg.value_type(arg) == types::F32 {
                    return Err(GlslError::new(
                        ErrorCode::E0301,
                        format!(
                            "F32 operand still present after fixed-point transformation: block = `{}`, inst = `{}`, operand[{}] = `{}` (type = F32)",
                            block,
                            func.dfg.display_inst(inst),
                            idx,
                            arg
                        ),
                    ));
                }
            }

            // Check branch arguments (values passed to blocks)
            for branch in func.dfg.insts[inst]
                .branch_destination(&func.dfg.jump_tables, &func.dfg.exception_tables)
            {
                for arg in branch.args(&func.dfg.value_lists) {
                    if let Some(val) = arg.as_value() {
                        if func.dfg.value_type(val) == types::F32 {
                            return Err(GlslError::new(
                                ErrorCode::E0301,
                                format!(
                                    "F32 value passed as branch argument: block = `{}`, inst = `{}`, value = `{}`",
                                    block,
                                    func.dfg.display_inst(inst),
                                    val
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Verify jump tables have no F32 values
fn verify_jump_tables(func: &Function) -> Result<(), GlslError> {
    use cranelift_codegen::ir::types;

    for jump_table in func.dfg.jump_tables.values() {
        for branch in jump_table.all_branches() {
            for arg in branch.args(&func.dfg.value_lists) {
                if let Some(val) = arg.as_value() {
                    if func.dfg.value_type(val) == types::F32 {
                        return Err(GlslError::new(
                            ErrorCode::E0301,
                            format!("F32 value in jump table branch: value = `{}`", val),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
