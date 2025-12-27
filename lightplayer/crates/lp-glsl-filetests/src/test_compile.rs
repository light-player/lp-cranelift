//! Compile test implementation.
//!
//! Tests CLIF IR output before any transformations (arch-agnostic).

use crate::file_update::FileUpdate;
use crate::validation::validate_clif_module;
use anyhow::{Context, Result};
use cranelift_codegen::ir::{ExternalName, UserFuncName};
use cranelift_codegen::write_function;
use cranelift_object::ObjectModule;
use lp_glsl::backend::module::gl_module::GlModule;
use lp_glsl::backend::target::Target;
use lp_glsl::GlslCompiler;
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Format a GlModule as CLIF text.
/// Exported for use by test_transform module.
pub fn format_clif_module(module: &GlModule<ObjectModule>) -> Result<String> {
    let mut result = String::new();

    // Build mapping from func_id string to function name for updating external references
    // In backend2, func_id_to_name is not stored, but we can build it from fns
    let mut name_mapping: HashMap<String, String> = HashMap::new();
    for (name, gl_func) in &module.fns {
        name_mapping.insert(gl_func.func_id.as_u32().to_string(), name.clone());
    }

    // Add user functions (excluding main)
    let mut user_funcs: Vec<_> = module
        .fns
        .iter()
        .filter(|(name, _)| *name != "main")
        .collect();
    // Sort by name for deterministic output
    user_funcs.sort_by_key(|(name, _)| *name);

    for (name, gl_func) in user_funcs {
        result.push_str(&format!("// function {}:\n", name));
        let func_text = format_function(&gl_func.function, name, &name_mapping)?;
        result.push_str(&func_text);
        result.push('\n');
    }

    // Add main function
    if let Some(main_func) = module.fns.get("main") {
        result.push_str("// function main:\n");
        let main_text = format_function(&main_func.function, "main", &name_mapping)?;
        result.push_str(&main_text);
    }

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
    // Skip compile test if source doesn't have a main() function
    // (compile tests require a complete shader with main())
    if !glsl_source.contains("main()") {
        return Ok(());
    }

    // Compile to GlModule (no transformations)
    let mut compiler = GlslCompiler::new();
    let target = Target::riscv32_emulator()
        .with_context(|| "failed to create riscv32 target for compile test")?;
    let module = compiler
        .compile_to_gl_module_object(glsl_source, target.clone())
        .with_context(|| "failed to compile GLSL to GlModule")?;

    // Get ISA for validation
    let mut target_for_isa = target.clone();
    let isa = target_for_isa
        .create_isa()
        .with_context(|| "failed to create ISA for validation")?;

    // Validate CLIF module
    validate_clif_module(&module, isa.as_ref())
        .with_context(|| "CLIF validation failed after compilation")?;

    // Extract CLIF text
    let actual_clif = format_clif_module(&module)?;

    // Compare with expectations
    compare_clif(&actual_clif, expected_clif, path, "compile")?;

    Ok(())
}

/// Format a single function as CLIF text with `//` prefix on each line.
/// The function is cloned and its name is set to the provided name, and external function
/// references are updated to use testcase names.
fn format_function(
    func: &cranelift_codegen::ir::Function,
    name: &str,
    name_mapping: &HashMap<String, String>,
) -> Result<String> {
    // Clone the function so we can modify it
    let mut func_clone = func.clone();

    // Set the function name to use testcase format (%name)
    func_clone.name = UserFuncName::testcase(name.as_bytes());

    // Update external function references to use testcase names
    // First collect the user_named_funcs mapping to avoid borrow conflicts
    let user_named_funcs: std::collections::HashMap<_, _> = func_clone
        .params
        .user_named_funcs()
        .iter()
        .map(|(k, v)| (k, v.clone()))
        .collect();
    for (_, ext_func) in func_clone.dfg.ext_funcs.iter_mut() {
        if let ExternalName::User(user_name_ref) = &ext_func.name {
            // Look up the function name from the user_named_funcs
            if let Some(user_name) = user_named_funcs.get(user_name_ref) {
                // The user_name.index should correspond to the func_id
                // Look up the name in the mapping
                if let Some(func_name) = name_mapping.get(&user_name.index.to_string()) {
                    ext_func.name = ExternalName::testcase(func_name.as_bytes());
                }
            }
        } else if let ExternalName::TestCase(_) = &ext_func.name {
            // Already using testcase format, leave as is
        } else if let ExternalName::LibCall(_) = &ext_func.name {
            // LibCall names are already in % format, leave as is
        }
    }

    let mut buf = String::new();
    write_function(&mut buf, &func_clone)
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
