//! Registry code generation.
//!
//! Generates Rust code for `BuiltinId` enum and `BuiltinRegistry` that maps
//! builtin IDs to their CLIF implementations.

use anyhow::Result;

/// Generate registry code from a list of builtin functions.
pub fn generate_registry(functions: &[BuiltinInfo], output_path: &str) -> Result<()> {
    // TODO: Implement in Phase 7
    Ok(())
}

/// Information about a builtin function for registry generation.
pub struct BuiltinInfo {
    /// Function name (e.g., `__lp_sqrt_u64`)
    pub name: String,
    
    /// Builtin ID (e.g., `SqrtU64`)
    pub id: String,
    
    /// Dependencies (other builtins this function calls)
    pub dependencies: Vec<String>,
}

