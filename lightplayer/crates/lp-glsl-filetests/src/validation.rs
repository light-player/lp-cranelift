//! CLIF validation utilities.

use anyhow::{Context, Result};
use cranelift_codegen::isa::TargetIsa;
use cranelift_codegen::print_errors::pretty_verifier_error;
use cranelift_codegen::verify_function;
use lp_glsl::ClifModule;

/// Validate all functions in a ClifModule
pub fn validate_clif_module(module: &ClifModule, isa: &dyn TargetIsa) -> Result<()> {
    // Validate user functions
    for (_, func) in module.user_functions() {
        verify_function(func, isa)
            .map_err(|errors| {
                anyhow::anyhow!(
                    "CLIF validation failed for function '{}':\n{}",
                    func.name,
                    pretty_verifier_error(func, None, errors)
                )
            })
            .with_context(|| format!("failed to validate function '{}'", func.name))?;
    }

    // Validate main function
    verify_function(module.main_function(), isa)
        .map_err(|errors| {
            anyhow::anyhow!(
                "CLIF validation failed for main function:\n{}",
                pretty_verifier_error(module.main_function(), None, errors)
            )
        })
        .with_context(|| "failed to validate main function")?;

    Ok(())
}
