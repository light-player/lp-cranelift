//! Run GLSL filetests
//! Pattern: cranelift/filetests/src/runone.rs

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn run_filetest(path: &Path) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    // Parse test directives
    let test_compile = source.contains("test compile");
    let test_run = source.contains("test run");
    
    if test_compile {
        crate::testing::test_compile::run_test(&source)?;
    }
    
    if test_run {
        crate::testing::test_run::run_test(&source)?;
    }
    
    if !test_compile && !test_run {
        anyhow::bail!("No test directives found (expected 'test compile' or 'test run')");
    }
    
    Ok(())
}

