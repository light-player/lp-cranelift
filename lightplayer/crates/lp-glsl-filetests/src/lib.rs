//! GLSL filetests infrastructure.
//!
//! This crate provides infrastructure for discovering, parsing, compiling, executing, and
//! verifying GLSL test files, matching Cranelift's filetests semantics.

#![deny(missing_docs)]

pub mod file_update;
pub mod filetest;
pub mod test_run;

use anyhow::Result;
use std::path::Path;

/// Run a single filetest.
pub fn run_filetest(path: &Path) -> Result<()> {
    let test_file = filetest::parse_test_file(path)?;
    test_run::run_test_file(&test_file, path)?;
    Ok(())
}
