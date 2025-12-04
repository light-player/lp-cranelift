//! Test CLIF IR generation
//! Pattern: cranelift/filetests/src/test_compile.rs

use anyhow::Result;
use filecheck::{CheckerBuilder, NO_VARIABLES};

pub fn run_test(full_source: &str, glsl_source: &str, fixed_point_format: Option<lp_glsl::FixedPointFormat>) -> Result<()> {
    // Compile GLSL to CLIF
    let mut jit = lp_glsl::JIT::new();
    jit.fixed_point_format = fixed_point_format;
    let clif = jit.compile_to_clif(glsl_source)
        .map_err(|e| anyhow::anyhow!("Compilation failed: {}", e))?;
    
    // Extract CHECK directives and run filecheck
    let checker = CheckerBuilder::new()
        .text(&clif)
        .map_err(|e| anyhow::anyhow!("Failed to build checker: {}", e))?
        .finish();
    
    checker
        .explain(full_source, NO_VARIABLES)
        .map_err(|e| anyhow::anyhow!("CHECK failed:\n{}", e))?;
    
    Ok(())
}

