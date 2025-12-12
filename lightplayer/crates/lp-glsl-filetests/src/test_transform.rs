//! Transform test implementation.
//!
//! Tests CLIF IR output after fixed32 transformation.

use crate::test_compile::{compare_clif, format_clif_module};
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
    let isa =
        create_riscv32_isa().with_context(|| "failed to create riscv32 ISA for transform test")?;
    let module = compiler
        .compile_to_clif_module(glsl_source, isa)
        .with_context(|| "failed to compile GLSL to CLIF module")?;

    // Apply fixed32 transformation
    let transformed_module = transform_module(&module, FixedPointFormat::Fixed16x16)
        .with_context(|| "failed to apply fixed32 transformation")?;

    // Extract CLIF text
    let actual_clif = format_clif_module(&transformed_module)?;

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path, "transform.fixed32")?;

    Ok(())
}

/// Create a riscv32 ISA for compilation.
fn create_riscv32_isa() -> Result<cranelift_codegen::isa::OwnedTargetIsa> {
    use cranelift_codegen::isa::riscv32::isa_builder;
    use cranelift_codegen::settings::{self, Configurable};
    use target_lexicon::{
        Architecture, BinaryFormat, Environment, OperatingSystem, Riscv32Architecture, Triple,
        Vendor,
    };

    let mut flag_builder = settings::builder();
    flag_builder
        .set("is_pic", "false")
        .map_err(|e| anyhow::anyhow!("failed to set is_pic: {}", e))?;
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|e| anyhow::anyhow!("failed to set use_colocated_libcalls: {}", e))?;
    flag_builder
        .set("enable_multi_ret_implicit_sret", "true")
        .map_err(|e| anyhow::anyhow!("failed to set enable_multi_ret_implicit_sret: {}", e))?;

    let flags = settings::Flags::new(flag_builder);
    let triple = Triple {
        architecture: Architecture::Riscv32(Riscv32Architecture::Riscv32imac),
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::None_,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf,
    };

    isa_builder(triple)
        .finish(flags)
        .map_err(|e| anyhow::anyhow!("failed to create riscv32 ISA: {}", e))
}
