//! Infrastructure for creating fixed-point math functions in CLIF modules.
//!
//! This module loads fixed-point math functions from CLIF text files
//! and inserts them into the module.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};

use cranelift_codegen::ir::{AbiParam, FuncRef, Function, Signature};
use cranelift_codegen::isa::CallConv;

#[cfg(feature = "cranelift-reader")]
use cranelift_reader::parse_functions;

/// Map to track created fixed-point functions by name
pub type FixedFunctionMap = HashMap<String, FuncRef>;

/// Storage for created function bodies that need to be added to the module
pub type CreatedFunctionBodies = HashMap<String, Function>;

/// Load CLIF file and return the parsed functions
#[cfg(feature = "cranelift-reader")]
fn load_clif_file(clif_text: &str) -> Result<Vec<Function>, GlslError> {
    parse_functions(clif_text).map_err(|e| {
        GlslError::new(
            ErrorCode::E0301,
            format!("Failed to parse CLIF file: {}", e),
        )
    })
}

#[cfg(not(feature = "cranelift-reader"))]
fn load_clif_file(_clif_text: &str) -> Result<Vec<Function>, GlslError> {
    Err(GlslError::new(
        ErrorCode::E0301,
        "CLIF file loading requires cranelift-reader feature",
    ))
}

/// Get or create a fixed-point sine function
///
/// This function checks if a sine function already exists, and if not,
/// loads it from a CLIF file and inserts it into the module.
pub fn get_or_create_sin_fixed(
    func: &mut Function,
    format: FixedPointFormat,
    created_functions: &mut FixedFunctionMap,
    created_bodies: &mut CreatedFunctionBodies,
) -> Result<FuncRef, GlslError> {
    // For Fixed16x16, we use fixed32/ directory (i32 types)
    // For Fixed32x32, we would use fixed64/ directory (i64 types) - future work
    let (func_name, clif_dir) = match format {
        FixedPointFormat::Fixed16x16 => ("lp_sin_fixed32", "fixed32"),
        FixedPointFormat::Fixed32x32 => {
            return Err(GlslError::new(
                ErrorCode::E0301,
                "Fixed32x32 format not yet supported - use Fixed16x16",
            ));
        }
    };

    // Check if function already exists in the map
    if let Some(&func_ref) = created_functions.get(func_name) {
        return Ok(func_ref);
    }

    // Load CLIF files - we need to load sin.clif and its dependencies
    // Parse all CLIF files and store all functions
    let reduce_angle_clif = include_str!("fixed/fixed32/reduce_angle.clif");
    let cordic_clif = include_str!("fixed/fixed32/cordic_rotation.clif");
    let sin_clif = include_str!("fixed/fixed32/sin.clif");

    // Parse all CLIF files
    let reduce_angle_funcs = load_clif_file(reduce_angle_clif)?;
    let cordic_funcs = load_clif_file(cordic_clif)?;
    let sin_funcs = load_clif_file(sin_clif)?;

    // Store all function bodies (dependencies first, then sin)
    for f in reduce_angle_funcs {
        let name = format!("{}", f.name);
        if !created_bodies.contains_key(&name) {
            created_bodies.insert(name, f);
        }
    }
    for f in cordic_funcs {
        let name = format!("{}", f.name);
        if !created_bodies.contains_key(&name) {
            created_bodies.insert(name, f);
        }
    }

    // Extract the sin function (should be the only one in sin_funcs)
    let sin_func = sin_funcs
        .into_iter()
        .next()
        .ok_or_else(|| GlslError::new(ErrorCode::E0301, "sin.clif did not contain a function"))?;

    // Create FuncRef for the sin function
    let sig_ref = func.import_signature(sin_func.signature.clone());
    let ext_func = cranelift_codegen::ir::ExtFuncData {
        name: cranelift_codegen::ir::ExternalName::testcase(func_name.as_bytes()),
        signature: sig_ref,
        colocated: false,
    };

    let func_ref = func.import_function(ext_func);

    // Store function body
    created_bodies.insert(String::from(func_name), sin_func);

    // Store in map
    created_functions.insert(String::from(func_name), func_ref);

    Ok(func_ref)
}
