//! GLSL filetests infrastructure.
//!
//! This crate provides infrastructure for discovering, parsing, compiling, executing, and
//! verifying GLSL test files, matching Cranelift's filetests semantics.

#![deny(missing_docs)]

pub mod file_update;
pub mod filetest;
pub mod filetest_parse;
pub mod test_compile;
pub mod test_run;
pub mod test_transform;
pub mod test_utils;

use anyhow::Result;
use std::path::Path;

/// Run a single filetest.
pub fn run_filetest(path: &Path) -> Result<()> {
    let test_file = filetest::parse_test_file(path)?;

    // Run compile test if requested
    if test_file.test_types.contains(&filetest::TestType::Compile) {
        test_compile::run_compile_test(
            &test_file.glsl_source,
            &test_file
                .clif_expectations
                .pre_transform
                .as_deref()
                .unwrap_or(""),
            path,
        )?;
    }

    // Run transform test if requested
    if test_file
        .test_types
        .contains(&filetest::TestType::TransformFixed32)
    {
        test_transform::run_transform_fixed32_test(
            &test_file.glsl_source,
            &test_file
                .clif_expectations
                .post_transform_fixed32
                .as_deref()
                .unwrap_or(""),
            path,
        )?;
    }

    // Run execution tests if requested
    if test_file
        .test_types
        .iter()
        .any(|t| matches!(t, filetest::TestType::Run))
    {
        test_run::run_test_file(&test_file, path)?;
    }

    Ok(())
}
