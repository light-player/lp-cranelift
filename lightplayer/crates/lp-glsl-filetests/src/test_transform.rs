//! Transform test implementation.
//!
//! Tests CLIF IR output after fixed32 transformation.

use crate::test_compile::{compare_clif, format_clif_module};
use crate::validation::validate_clif_module;
use anyhow::{Context, Result};
use lp_glsl::backend::target::Target;
use lp_glsl::backend::transform::fixed32::{Fixed32Transform, FixedPointFormat};
use lp_glsl::GlslCompiler;
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

    // Compile to GlModule
    let mut compiler = GlslCompiler::new();
    let target = Target::riscv32_emulator()
        .with_context(|| "failed to create riscv32 target for transform test")?;
    let module = compiler
        .compile_to_gl_module_object(glsl_source, target.clone())
        .with_context(|| "failed to compile GLSL to GlModule")?;

    // Apply fixed32 transformation
    let transform = Fixed32Transform::new(FixedPointFormat::Fixed16x16);
    let transformed_module = module
        .apply_transform(transform)
        .with_context(|| "failed to apply fixed32 transformation")?;

    // Get ISA for validation
    let mut target_for_isa = target.clone();
    let isa = target_for_isa
        .create_isa()
        .with_context(|| "failed to create ISA for validation")?;

    // Validate CLIF module after transformation
    validate_clif_module(&transformed_module, isa.as_ref())
        .with_context(|| "CLIF validation failed after fixed32 transformation")?;

    // Extract CLIF text
    let actual_clif = format_clif_module(&transformed_module)?;

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path, "transform.fixed32")?;

    Ok(())
}
