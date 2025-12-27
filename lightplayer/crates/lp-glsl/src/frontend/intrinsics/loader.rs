//! Intrinsic function loader - manages compilation and caching of intrinsic functions.

use crate::error::{ErrorCode, GlslError};
use crate::frontend::codegen::context::CodegenContext;
use cranelift_codegen::{
    Context,
    ir::{FuncRef, Function},
};
use cranelift_module::{FuncOrDataId, Linkage};

use alloc::string::String;
use hashbrown::HashMap;

use super::compiler::compile_intrinsic_functions;

/// Cache for compiled intrinsic functions per module.
pub struct IntrinsicCache {
    /// Compiled function bodies (function name -> Function object)
    pub compiled_functions: HashMap<String, Function>,
    /// Function references in the module (function name -> FuncRef)
    pub module_func_refs: HashMap<String, FuncRef>,
}

impl IntrinsicCache {
    pub fn new() -> Self {
        Self {
            compiled_functions: HashMap::new(),
            module_func_refs: HashMap::new(),
        }
    }
}

/// Map library call name to intrinsic function name.
fn map_to_intrinsic_name(libcall_name: &str) -> Result<&str, GlslError> {
    match libcall_name {
        "sinf" => Ok("__lp_sin"),
        "cosf" => Ok("__lp_cos"),
        "tanf" => Ok("__lp_tan"),
        "asinf" => Ok("__lp_asin"),
        "acosf" => Ok("__lp_acos"),
        "atanf" => Ok("__lp_atan"),
        "sinhf" => Ok("__lp_sinh"),
        "coshf" => Ok("__lp_cosh"),
        "tanhf" => Ok("__lp_tanh"),
        "asinhf" => Ok("__lp_asinh"),
        "acoshf" => Ok("__lp_acosh"),
        "atanhf" => Ok("__lp_atanh"),
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            format!("Unknown math function: {}", libcall_name),
        )),
    }
}

/// Determine which GLSL file contains the intrinsic function.
fn get_intrinsic_file(intrinsic_name: &str) -> Result<&str, GlslError> {
    match intrinsic_name {
        "__lp_sin" | "__lp_cos" | "__lp_tan" | "__lp_asin" | "__lp_acos" | "__lp_atan"
        | "__lp_sinh" | "__lp_cosh" | "__lp_tanh" | "__lp_asinh" | "__lp_acosh" | "__lp_atanh" => {
            Ok("trig")
        }
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            format!("Unknown intrinsic function: {}", intrinsic_name),
        )),
    }
}

/// Get or create an intrinsic function reference.
///
/// This function checks the cache, and if the function doesn't exist,
/// loads and compiles the appropriate GLSL file, then declares the function
/// in the module.
pub fn get_or_create_intrinsic(
    libcall_name: &str,
    ctx: &mut CodegenContext,
) -> Result<FuncRef, GlslError> {
    let source_map = &mut ctx.source_map;
    // Note: current_file_id is available via ctx.current_file_id if needed for error context
    // Map library call name to intrinsic name
    let intrinsic_name = map_to_intrinsic_name(libcall_name)?;

    // Get or create cache
    let cache = ctx
        .intrinsic_cache
        .as_mut()
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, "Intrinsic cache not initialized"))?;

    // Check if function already exists in module
    if let Some(&func_ref) = cache.module_func_refs.get(intrinsic_name) {
        return Ok(func_ref);
    }

    // Determine which GLSL file to load
    let file_name = get_intrinsic_file(intrinsic_name)?;

    // Check if intrinsic file is already in source map
    let intrinsic_file_id = if let Some(existing_id) = source_map.find_intrinsic(file_name) {
        existing_id
    } else {
        // Load and add intrinsic file to source map
        let glsl_source = match file_name {
            "trig" => include_str!("trig.glsl"),
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("Unknown intrinsic file: {}", file_name),
                ));
            }
        };
        source_map.add_file(
            crate::frontend::src_loc::GlFileSource::Intrinsic(String::from(file_name)),
            String::from(glsl_source),
        )
    };

    // Compile all functions in the GLSL file
    let isa = ctx.module.isa();
    // Extract source string to avoid borrow conflicts (clone to release immutable borrow)
    let glsl_source_str = source_map.get_file(intrinsic_file_id)
        .ok_or_else(|| GlslError::new(
            ErrorCode::E0400,
            format!("Intrinsic file {} not found in source map", file_name),
        ))?
        .contents.clone();
    let compiled_functions = compile_intrinsic_functions(
        glsl_source_str.as_str(),
        isa,
        source_map,
        intrinsic_file_id,
    )?;

    // Declare and define all functions first (they may call each other)
    let mut func_ids = hashbrown::HashMap::new();
    for (name, func) in &compiled_functions {
        // Check if function is already declared in module
        let func_id = if let Some(FuncOrDataId::Func(id)) = ctx.module.get_name(name) {
            // Function already declared, use existing ID
            id
        } else {
            // Declare function in module
            ctx.module
                .declare_function(name, Linkage::Local, &func.signature)
                .map_err(|e| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("Failed to declare intrinsic function {}: {}", name, e),
                    )
                })?
        };

        func_ids.insert(name.clone(), func_id);
    }

    // Define all functions that aren't already in cache
    for (name, func) in &compiled_functions {
        if !cache.compiled_functions.contains_key(name) {
            if let Some(&func_id) = func_ids.get(name) {
                // Create a Context from the compiled function
                let mut func_ctx = Context::for_function(func.clone());

                // Define the function in the module
                ctx.module
                    .define_function(func_id, &mut func_ctx)
                    .map_err(|e| {
                        GlslError::new(
                            ErrorCode::E0400,
                            format!("Failed to define intrinsic function {}: {}", name, e),
                        )
                    })?;
            }
        }
    }

    // Store all compiled functions in cache
    for (name, func) in compiled_functions {
        if !cache.compiled_functions.contains_key(&name) {
            cache.compiled_functions.insert(name.clone(), func);
        }
    }

    // Get the requested function's ID
    let func_id = func_ids.get(intrinsic_name).ok_or_else(|| {
        GlslError::new(
            ErrorCode::E0400,
            format!(
                "Intrinsic function {} not found in compiled functions",
                intrinsic_name
            ),
        )
    })?;

    // Import into current function
    let func_ref = ctx.module.declare_func_in_func(*func_id, ctx.builder.func);

    // Store in cache
    cache
        .module_func_refs
        .insert(String::from(intrinsic_name), func_ref);

    Ok(func_ref)
}
