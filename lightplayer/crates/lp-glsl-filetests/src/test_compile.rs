//! Compile test implementation.
//!
//! Tests CLIF IR output before any transformations (arch-agnostic).

use crate::file_update::FileUpdate;
use anyhow::{Context, Result};
use cranelift_codegen::write_function;
use lp_glsl::{ClifModule, GlslCompiler};
use std::env;
use std::path::Path;

/// Format a ClifModule as CLIF text.
/// Exported for use by test_transform module.
pub fn format_clif_module(module: &ClifModule) -> Result<String> {
    let mut result = String::new();

    // Add user functions
    let mut user_funcs: Vec<_> = module.user_functions().iter().collect();
    // Sort by name for deterministic output
    user_funcs.sort_by_key(|(name, _)| *name);

    for (name, func) in user_funcs {
        result.push_str(&format!("// function {}:\n", name));
        let func_text = format_function(func)?;
        result.push_str(&func_text);
        result.push('\n');
    }

    // Add main function
    result.push_str("// function main:\n");
    let main_text = format_function(module.main_function())?;
    result.push_str(&main_text);

    Ok(result)
}

/// Compare actual CLIF with expected CLIF.
/// Exported for use by test_transform module.
pub fn compare_clif(actual: &str, expected: &str, path: &Path, test_type: &str) -> Result<()> {
    let bless_enabled = env::var("CRANELIFT_TEST_BLESS").unwrap_or_default() == "1";

    // Normalize both strings
    let actual_normalized = normalize_clif(actual);
    let expected_normalized = normalize_clif(expected);

    if actual_normalized != expected_normalized {
        if bless_enabled {
            // Update expectations in file
            let file_update = FileUpdate::new(path);
            file_update.update_clif_expectations(test_type, &actual_normalized)?;
        } else {
            // Show diff and suggest bless mode
            anyhow::bail!(
                "CLIF mismatch in {} test:\n\n{}\n\n\
                 This test assertion can be automatically updated by setting the\n\
                 CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                test_type,
                format_diff(&expected_normalized, &actual_normalized)
            );
        }
    }

    Ok(())
}

/// Run a compile test: verify CLIF IR before transformations.
pub fn run_compile_test(glsl_source: &str, expected_clif: &str, path: &Path) -> Result<()> {
    // Compile to CLIF (no transformations)
    let mut compiler = GlslCompiler::new();
    let isa =
        create_riscv32_isa().with_context(|| "failed to create riscv32 ISA for compile test")?;
    let module = compiler
        .compile_to_clif_module(glsl_source, isa)
        .with_context(|| "failed to compile GLSL to CLIF module")?;

    // Extract CLIF text
    let actual_clif = format_clif_module(&module)?;

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path, "compile")?;

    Ok(())
}

/// Format a single function as CLIF text with `//` prefix on each line.
fn format_function(func: &cranelift_codegen::ir::Function) -> Result<String> {
    let mut buf = String::new();
    write_function(&mut buf, func)
        .map_err(|e| anyhow::anyhow!("failed to write function: {}", e))?;

    // Prefix each line with "// "
    Ok(buf
        .lines()
        .map(|line| format!("// {}", line))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Normalize CLIF text for comparison.
fn normalize_clif(clif: &str) -> String {
    clif.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() || line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a diff between expected and actual CLIF.
fn format_diff(expected: &str, actual: &str) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    writeln!(result, "Expected:").ok();
    writeln!(result, "{}", expected).ok();
    writeln!(result, "\nActual:").ok();
    writeln!(result, "{}", actual).ok();
    result
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
