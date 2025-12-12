//! Transform test implementation.
//!
//! Tests CLIF IR output after fixed32 transformation.

use crate::test_compile::{compare_clif, format_clif_module};
use crate::test_utils;
use crate::validation::validate_clif_module;
use anyhow::{Context, Result};
use lp_glsl::{FixedPointFormat, GlslCompiler, transform_module};
use std::path::Path;

/// Run a transform test: verify CLIF IR after fixed32 transformation.
pub fn run_transform_fixed32_test(
    glsl_source: &str,
    expected_clif: &str,
    path: &Path,
) -> Result<()> {
    // Skip transform test if source doesn't have a main() function
    // (transform tests require a complete shader with main())
    if !glsl_source.contains("main()") {
        return Ok(());
    }

    // Compile to CLIF
    let mut compiler = GlslCompiler::new();
    let isa = test_utils::create_riscv32_isa()
        .with_context(|| "failed to create riscv32 ISA for transform test")?;
    let module = compiler
        .compile_to_clif_module(glsl_source, isa.clone())
        .with_context(|| "failed to compile GLSL to CLIF module")?;

    // Apply fixed32 transformation
    let transformed_module = transform_module(&module, FixedPointFormat::Fixed16x16)
        .with_context(|| "failed to apply fixed32 transformation")?;

    // Validate CLIF module after transformation
    validate_clif_module(&transformed_module, isa.as_ref())
        .with_context(|| "CLIF validation failed after fixed32 transformation")?;

    // Extract CLIF text
    let actual_clif = format_clif_module(&transformed_module)?;

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path, "transform.fixed32")?;

    Ok(())
}
