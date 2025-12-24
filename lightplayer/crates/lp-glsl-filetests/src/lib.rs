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
use glob::{glob_with, MatchOptions};

/// Run a single filetest.
pub fn run_filetest(path: &Path) -> Result<()> {
    run_filetest_with_line_filter(path, None, true)
}

/// Run a single filetest with optional line number filtering.
pub fn run_filetest_with_line_filter(path: &Path, line_filter: Option<usize>, show_full_output: bool) -> Result<()> {
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
        test_run::run_test_file_with_line_filter(&test_file, path, line_filter, show_full_output)?;
    }

    Ok(())
}

/// Represents a parsed file path that may include a line number.
#[derive(Debug, Clone)]
struct FileSpec {
    path: PathBuf,
    line_number: Option<usize>,
}


/// Check if a string contains glob pattern characters
fn contains_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

/// Expand glob patterns and return matching file paths
fn expand_glob_patterns(pattern: &str, filetests_dir: &Path) -> Result<Vec<PathBuf>> {
    let full_pattern = if pattern.contains('/') {
        // Pattern contains path separators, use as-is (e.g., "math/*" -> "filetests/math/*")
        filetests_dir.join(pattern)
    } else {
        // Pattern is just a filename pattern, search recursively (e.g., "*add*" -> "filetests/**/*.glsl" with name filtering)
        filetests_dir.join("**").join("*.glsl")
    };

    // Convert to string for glob crate
    let pattern_str = full_pattern.to_string_lossy();

    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    let mut matches = Vec::new();
    for entry in glob_with(&pattern_str, options)? {
        match entry {
            Ok(path) => {
                // Only include .glsl files
                if path.extension().and_then(|s| s.to_str()) == Some("glsl") {
                    // For filename-only patterns, also check if the filename matches the pattern
                    let should_include = if pattern.contains('/') {
                        // Path pattern - already matched by glob
                        true
                    } else {
                        // Filename pattern - check if filename matches the pattern
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|name| {
                                if contains_glob_pattern(pattern) {
                                    // Has glob characters - do glob-style matching
                                    // For now, simple substring match (could use regex for full glob support)
                                    let pattern_no_stars = pattern.trim_matches('*');
                                    name.contains(pattern_no_stars)
                                } else {
                                    // Exact filename match
                                    name == pattern
                                }
                            })
                            .unwrap_or(false)
                    };

                    if should_include {
                        matches.push(path);
                    }
                }
            }
            Err(e) => {
                // Log warning but continue
                eprintln!("Warning: glob pattern error: {}", e);
            }
        }
    }

    // Sort for deterministic output
    matches.sort();
    Ok(matches)
}

/// Parse a file specification that may contain glob patterns and line numbers
fn parse_file_spec_with_glob(file_str: &str, filetests_dir: &Path) -> Result<Vec<FileSpec>> {
    // Check if it contains a line number (format: pattern:line_number)
    let (pattern, line_number) = if let Some(colon_pos) = file_str.find(':') {
        let (pattern_part, line_part) = file_str.split_at(colon_pos);
        let line_str = &line_part[1..]; // Skip the colon

        match line_str.parse::<usize>() {
            Ok(line) => (pattern_part, Some(line)),
            Err(_) => {
                // Not a valid line number, treat whole string as pattern
                (file_str, None)
            }
        }
    } else {
        (file_str, None)
    };

    // Check if pattern contains glob characters or is a filename (no path separators)
    let paths = if contains_glob_pattern(pattern) || !pattern.contains('/') {
        expand_glob_patterns(pattern, filetests_dir)?
    } else {
        // Pattern contains path separators and no glob characters, treat as literal path
        let full_path = filetests_dir.join(pattern);
        if full_path.exists() {
            vec![full_path]
        } else {
            vec![]
        }
    };

    // Create FileSpec for each matching path
    let mut specs = Vec::new();
    for path in paths {
        specs.push(FileSpec {
            path,
            line_number,
        });
    }

    Ok(specs)
}

/// Main entry point for `lp-test test`.
///
/// Take a list of filenames which can be either `.glsl` files or directories.
/// Files can optionally include line numbers in the format `file.glsl:42`.
/// Glob patterns are supported (e.g., `*.glsl`, `math/*`, `*add*`).
///
/// Files are interpreted as test cases and executed immediately.
///
/// Directories are scanned recursively for test cases ending in `.glsl`.
pub fn run(verbose: bool, files: &[String]) -> anyhow::Result<()> {
    let filetests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");
    let mut test_specs = Vec::new();

    // Parse all file specifications, expanding glob patterns as needed
    for file_str in files {
        let specs = parse_file_spec_with_glob(file_str, &filetests_dir)?;

        for spec in specs {
            // Validate that the path exists and is a .glsl file
            if spec.path.is_file() {
                if spec.path.extension().and_then(|s| s.to_str()) == Some("glsl") {
                    test_specs.push(spec);
                } else {
                    eprintln!("Warning: {} is not a .glsl file, skipping", spec.path.display());
                }
            } else {
                eprintln!("Warning: {} does not exist or is not a file, skipping", spec.path.display());
            }
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

    // Determine if we're running a single test (show full output) or multiple tests (suppress verbose output)
    let show_full_output = test_specs.len() == 1;

    for spec in &test_specs {
        let display_path = if let Some(line) = spec.line_number {
            format!("{}:{}", spec.path.display(), line)
        } else {
            spec.path.display().to_string()
        };

        if verbose {
            println!("Running test: {}", display_path);
        }

        match run_filetest_with_line_filter(&spec.path, spec.line_number, show_full_output) {
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
