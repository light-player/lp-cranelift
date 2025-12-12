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
pub mod validation;

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Run a single filetest.
pub fn run_filetest(path: &Path) -> Result<()> {
    run_filetest_with_line_filter(path, None)
}

/// Run a single filetest with optional line number filtering.
pub fn run_filetest_with_line_filter(path: &Path, line_filter: Option<usize>) -> Result<()> {
    let test_file = filetest::parse_test_file(path)?;

    // Validate line number if provided
    if let Some(line_number) = line_filter {
        let has_matching_directive = test_file
            .run_directives
            .iter()
            .any(|directive| directive.line_number == line_number);
        if !has_matching_directive {
            anyhow::bail!(
                "line {} does not contain a valid run directive",
                line_number
            );
        }
    }

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
        test_run::run_test_file_with_line_filter(&test_file, path, line_filter)?;
    }

    Ok(())
}

/// Represents a parsed file path that may include a line number.
#[derive(Debug, Clone)]
struct FileSpec {
    path: PathBuf,
    line_number: Option<usize>,
}

impl FileSpec {
    /// Parse a file specification like "path/to/file.glsl:42" or "path/to/file.glsl"
    fn parse(file_str: &str) -> anyhow::Result<Self> {
        let path = Path::new(file_str);

        // Check if the path contains a line number (format: path:line_number)
        if let Some(path_str) = file_str.split(':').next() {
            let actual_path = Path::new(path_str);
            if let Some(line_part) = file_str
                .strip_prefix(path_str)
                .and_then(|s| s.strip_prefix(':'))
            {
                if let Ok(line_number) = line_part.parse::<usize>() {
                    return Ok(FileSpec {
                        path: actual_path.to_path_buf(),
                        line_number: Some(line_number),
                    });
                } else {
                    anyhow::bail!("invalid line number '{}' in '{}'", line_part, file_str);
                }
            }
        }

        Ok(FileSpec {
            path: path.to_path_buf(),
            line_number: None,
        })
    }
}

/// Main entry point for `lp-test test`.
///
/// Take a list of filenames which can be either `.glsl` files or directories.
/// Files can optionally include line numbers in the format `file.glsl:42`.
///
/// Files are interpreted as test cases and executed immediately.
///
/// Directories are scanned recursively for test cases ending in `.glsl`.
pub fn run(verbose: bool, files: &[String]) -> anyhow::Result<()> {
    let mut test_specs = Vec::new();

    // Parse all file specifications and collect .glsl files
    for file_str in files {
        let spec = FileSpec::parse(file_str)?;

        if spec.path.is_file() {
            if spec.path.extension().and_then(|s| s.to_str()) == Some("glsl") {
                test_specs.push(spec);
            } else {
                eprintln!("Warning: {} is not a .glsl file, skipping", file_str);
            }
        } else if spec.path.is_dir() {
            // Scan directory for .glsl files
            for entry in WalkDir::new(&spec.path) {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.extension().and_then(|s| s.to_str()) == Some("glsl") {
                    test_specs.push(FileSpec {
                        path: entry_path.to_path_buf(),
                        line_number: spec.line_number, // Apply line filter to all files in directory
                    });
                }
            }
        } else {
            eprintln!("Warning: {} does not exist, skipping", file_str);
        }
    }

    if test_specs.is_empty() {
        anyhow::bail!("no .glsl test files found");
    }

    // Sort for deterministic output
    test_specs.sort_by(|a, b| a.path.cmp(&b.path));

    let mut passed = 0;
    let mut failed = 0;

    println!("Running {} test file(s)...\n", test_specs.len());

    for spec in &test_specs {
        let display_path = if let Some(line) = spec.line_number {
            format!("{}:{}", spec.path.display(), line)
        } else {
            spec.path.display().to_string()
        };

        if verbose {
            println!("Running test: {}", display_path);
        }

        match run_filetest_with_line_filter(&spec.path, spec.line_number) {
            Ok(()) => {
                println!("✓ {}", display_path);
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}: {}", display_path, e);
                if verbose {
                    println!("  Error details: {:#}", e);
                }
                failed += 1;
            }
        }
    }

    println!("\nResults: {} passed, {} failed", passed, failed);

    if failed > 0 {
        anyhow::bail!("{} test file(s) failed", failed);
    }

    Ok(())
}
