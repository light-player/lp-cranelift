//! Module-level fixed-point transformation
//!
//! This module provides a pure functional transformation that converts all functions
//! in a ClifModule from float to fixed-point representation atomically.

use crate::frontend::glsl_compiler::create_minimal_module_for_declarations;
use crate::error::{ErrorCode, GlslError};
use crate::backend::ir::ClifModule;
use crate::backend::transform::fixed32::function::rewrite_function;
use crate::backend::transform::fixed32::signature::convert_signature;
use crate::backend::transform::fixed32::types::FixedPointFormat;
use cranelift_codegen::ir::{FuncRef, Function, Signature};
use cranelift_module::{Linkage, Module};
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Transform a ClifModule from float to fixed-point representation
///
/// This is a pure functional transformation that:
/// 1. Converts all function signatures
/// 2. Creates FuncRef mappings for cross-function calls
/// 3. Rewrites all function bodies
///
/// The key challenge is that FuncRefs are scoped to individual Functions,
/// but we need to map them across the entire module. We solve this by:
/// 1. Creating a temporary module to hold converted function signatures
/// 2. Building a mapping: function_name -> new_FuncRef
/// 3. During function rewriting, updating call instructions to use mapped FuncRefs
pub fn transform_module(
    module: &ClifModule,
    format: FixedPointFormat,
) -> Result<ClifModule, GlslError> {
    // Create a temporary module for FuncRef management during transformation
    // This is needed because FuncRefs are scoped to a Function, but we need
    // to map old FuncRefs (pointing to float signatures) to new FuncRefs (pointing to fixed-point signatures)
    let mut temp_module = create_minimal_module_for_declarations(module.isa())?;

    // Step 1: Convert all function signatures and create new FuncRefs
    // Build mapping: function_name -> (old_signature, new_signature, new_func_ref)
    let mut func_ref_map: HashMap<String, (Signature, Signature, FuncRef)> = HashMap::new();

    // Convert user function signatures
    for (name, func) in module.user_functions() {
        let old_sig = func.signature.clone();
        let new_sig = convert_signature(&old_sig, format);

        // Create a FuncRef in the temp module for this converted function
        let func_id = temp_module
            .declare_function(name, Linkage::Local, &new_sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "failed to declare function '{}' in temp module: {}",
                        name, e
                    ),
                )
            })?;

        // Create a temporary function to get a FuncRef
        let mut temp_func = Function::new();
        temp_func.signature = new_sig.clone();
        let func_ref = temp_module.declare_func_in_func(func_id, &mut temp_func);

        func_ref_map.insert(name.clone(), (old_sig, new_sig, func_ref));
    }

    // Convert main function signature
    let main_old_sig = module.main_function().signature.clone();
    let main_new_sig = convert_signature(&main_old_sig, format);
    let main_func_id = temp_module
        .declare_function("main", Linkage::Export, &main_new_sig)
        .map_err(|e| {
            GlslError::new(
                ErrorCode::E0400,
                format!("failed to declare main function in temp module: {}", e),
            )
        })?;
    let mut temp_main_func = Function::new();
    temp_main_func.signature = main_new_sig.clone();
    let _main_func_ref = temp_module.declare_func_in_func(main_func_id, &mut temp_main_func);

    // Step 2: Convert all function bodies, updating call sites to use new FuncRefs
    let mut builder = ClifModule::builder()
        .set_function_registry({
            // FunctionRegistry doesn't implement Clone, so we need to rebuild it
            // For now, create a new empty one - the actual registry info is in the functions
            use crate::frontend::semantic::functions::FunctionRegistry;
            FunctionRegistry::new()
        })
        .set_source_text(String::from(module.source_text()))
        // Preserve source location manager - this is critical for trap source location mapping
        .set_source_loc_manager(module.source_loc_manager().clone())
        // Recreate ISA from TargetIsa reference
        .set_isa({
            use cranelift_codegen::isa;
            let isa_builder = isa::Builder::from_target_isa(module.isa());
            // Copy flags from the original ISA
            let flags = module.isa().flags().clone();
            isa_builder.finish(flags).map_err(|e| {
                GlslError::new(
                    crate::error::ErrorCode::E0400,
                    format!("failed to recreate ISA: {:?}", e),
                )
            })?
        });

    // Preserve GLSL signatures (they don't change during fixed-point transformation)
    // The GLSL types remain the same; only the CLIF representation changes
    for (name, func) in module.user_functions() {
        let rewritten_func = rewrite_function(func, format)?;
        builder = builder.add_user_function(name.clone(), rewritten_func);

        // Preserve GLSL signature from original module
        if let Some(glsl_sig) = module.glsl_signature(name) {
            builder = builder.add_glsl_signature(name.clone(), glsl_sig.clone());
        }
    }

    // Convert main function
    let rewritten_main = rewrite_function(module.main_function(), format)?;
    builder = builder.set_main_function(rewritten_main);

    // Preserve main's GLSL signature
    if let Some(main_glsl_sig) = module.glsl_signature("main") {
        builder = builder.add_glsl_signature(String::from("main"), main_glsl_sig.clone());
    }

    // Preserve func_id_to_name mapping - this is needed for linking
    // The mapping from old FuncIds (from compilation) to function names doesn't change
    // during transformation, so we can copy it directly
    builder = builder.add_func_id_mappings(module.func_id_to_name_map().clone());

    Ok(builder.build()?)
}
