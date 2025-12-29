//! GLSL filetests infrastructure.
//!
//! This crate provides infrastructure for discovering, parsing, compiling, executing, and
//! verifying GLSL test files, matching Cranelift's filetests semantics.

#![deny(missing_docs)]

pub mod concurrent;
pub mod file_update;
pub mod filetest;
pub mod filetest_parse;
pub mod test_compile;
pub mod test_run;
pub mod test_transform;
pub mod test_utils;
pub mod validation;

use anyhow::Result;
use glob::{MatchOptions, glob_with};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

/// ANSI color codes for terminal output (matching Rust's test output style)
mod colors {
    pub const GREEN: &str = "\x1b[32m";
    pub const RED: &str = "\x1b[31m";
    pub const DIM: &str = "\x1b[2m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// Check if colors should be enabled.
/// Respects NO_COLOR environment variable.
fn should_color() -> bool {
    std::env::var("NO_COLOR").is_err()
}

/// Format text with color if colors are enabled.
fn colorize(text: &str, color: &str) -> String {
    if should_color() {
        format!("{}{}{}", color, text, colors::RESET)
    } else {
        text.to_string()
    }
}

/// Compute relative path from filetests_dir to the given path.
fn relative_path(path: &Path, filetests_dir: &Path) -> String {
    path.strip_prefix(filetests_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Run a single filetest.
pub fn run_filetest(path: &Path) -> Result<()> {
    run_filetest_with_line_filter(path, None, true)
}

/// Run a single filetest with optional line number filtering.
pub fn run_filetest_with_line_filter(
    path: &Path,
    line_filter: Option<usize>,
    show_full_output: bool,
) -> Result<()> {
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

/// Represents a failed test for summary reporting.
struct FailedTest {
    path: PathBuf,
    line_number: Option<usize>,
}

/// Test execution state for tracking parallel execution.
#[derive(Debug)]
enum TestState {
    New,
    Queued,
    Done(anyhow::Result<()>),
}

/// Test entry for parallel execution.
struct TestEntry {
    spec: FileSpec,
    state: TestState,
}

/// Check if a string contains glob pattern characters
fn contains_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

/// Expand glob patterns and return matching paths (files or directories)
fn expand_glob_patterns(pattern: &str, filetests_dir: &Path) -> Result<Vec<PathBuf>> {
    // Build the glob pattern - append pattern to filetests_dir
    // If pattern doesn't contain '/', it will match files/directories at the top level
    // If pattern contains '/', it will match at that specific path level
    let full_pattern = filetests_dir.join(pattern);

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
                // Include both files and directories - directories will be handled later
                // to recursively find all .glsl files
                if path.is_file() {
                    // Only include .glsl files
                    if path.extension().and_then(|s| s.to_str()) == Some("glsl") {
                        matches.push(path);
                    }
                } else if path.is_dir() {
                    // Include directories - they'll be expanded to find all .glsl files
                    matches.push(path);
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

    // Check if pattern contains glob characters
    let paths = if contains_glob_pattern(pattern) {
        // Use glob to expand the pattern - this will match files and directories
        expand_glob_patterns(pattern, filetests_dir)?
    } else {
        // No glob characters - treat as literal path
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
        specs.push(FileSpec { path, line_number });
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
///
/// Mode is determined by test count:
/// - Single test (1 file): Full detailed output with all error information
/// - Multiple tests (>1 file): Minimal output with colored checkmarks
pub fn run(files: &[String]) -> anyhow::Result<()> {
    let filetests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");
    let mut test_specs = Vec::new();

    // Parse all file specifications, expanding glob patterns as needed
    for file_str in files {
        let specs = parse_file_spec_with_glob(file_str, &filetests_dir)?;

        for spec in specs {
            // Handle directories by recursively finding all .glsl files
            if spec.path.is_dir() {
                for entry in WalkDir::new(&spec.path) {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file()
                                && path.extension().and_then(|s| s.to_str()) == Some("glsl")
                            {
                                test_specs.push(FileSpec {
                                    path: path.to_path_buf(),
                                    line_number: spec.line_number,
                                });
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: error walking directory {}: {}",
                                spec.path.display(),
                                e
                            );
                        }
                    }
                }
            } else if spec.path.is_file() {
                // Validate that the path exists and is a .glsl file
                if spec.path.extension().and_then(|s| s.to_str()) == Some("glsl") {
                    test_specs.push(spec);
                } else {
                    eprintln!(
                        "Warning: {} is not a .glsl file, skipping",
                        spec.path.display()
                    );
                }
            } else {
                eprintln!("Warning: {} does not exist, skipping", spec.path.display());
            }
        }
    }

    if test_specs.is_empty() {
        anyhow::bail!("no .glsl test files found");
    }

    // Sort for deterministic output
    test_specs.sort_by(|a, b| a.path.cmp(&b.path));

    println!("Running {} test file(s)...\n", test_specs.len());

    // Start timing
    let start_time = Instant::now();

    // Determine if we're running a single test (show full output) or multiple tests (minimal output)
    let show_full_output = test_specs.len() == 1;

    // Use parallel execution for multiple tests, sequential for single test
    if test_specs.len() == 1 {
        // Single test: run sequentially and show full details
        let spec = &test_specs[0];
        let relative_path_str = relative_path(&spec.path, &filetests_dir);
        let display_path = if let Some(line) = spec.line_number {
            format!("{}:{}", relative_path_str, line)
        } else {
            relative_path_str
        };

        // Catch panics in single-test mode too
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_filetest_with_line_filter(&spec.path, spec.line_number, show_full_output)
        }))
        .unwrap_or_else(|e| {
            // Convert panic to error
            let panic_msg = if let Some(msg) = e.downcast_ref::<String>() {
                msg.clone()
            } else if let Some(msg) = e.downcast_ref::<&'static str>() {
                msg.to_string()
            } else {
                format!("{:?}", e)
            };
            anyhow::bail!("panicked: {}", panic_msg)
        });

        let (passed, failed) = match result {
            Ok(()) => {
                // Single test passed - show success with color
                println!(
                    "{}",
                    colorize(&format!("✓ {}", display_path), colors::GREEN)
                );
                (1, 0)
            }
            Err(e) => {
                // Single test failed - show failure marker and full error details
                println!("{}", colorize(&format!("✗ {}", display_path), colors::RED));
                println!("\n{:#}", e);
                (0, 1)
            }
        };

        let elapsed = start_time.elapsed();
        println!("\n{}", format_results_summary(passed, failed, elapsed));

        if failed > 0 {
            println!(
                "\n{}",
                colorize("Tip: Rerun with DEBUG=1 for detailed logging", colors::DIM)
            );
            anyhow::bail!("{} test file(s) failed", failed);
        }

        return Ok(());
    }

    // Multiple tests: use parallel execution
    let mut tests: Vec<TestEntry> = test_specs
        .into_iter()
        .map(|spec| TestEntry {
            spec,
            state: TestState::New,
        })
        .collect();

    let mut concurrent_runner = concurrent::ConcurrentRunner::new();
    let mut next_test = 0;
    let mut reported_tests = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    // Queue all tests
    while next_test < tests.len() {
        let jobid = next_test;
        tests[jobid].state = TestState::Queued;
        concurrent_runner.put(
            jobid,
            &tests[jobid].spec.path,
            tests[jobid].spec.line_number,
            show_full_output,
        );
        next_test += 1;
    }

    // Process replies and report results in order
    while reported_tests < tests.len() {
        // Check for completed jobs
        while let Some(reply) = concurrent_runner.try_get() {
            match reply {
                concurrent::Reply::Done { jobid, result } => {
                    tests[jobid].state = TestState::Done(result);
                }
            }
        }

        // Report next test in order if it's done
        if reported_tests < tests.len() {
            if let TestState::Done(ref result) = tests[reported_tests].state {
                let spec = &tests[reported_tests].spec;
                let relative_path_str = relative_path(&spec.path, &filetests_dir);
                let display_path = if let Some(line) = spec.line_number {
                    format!("{}:{}", relative_path_str, line)
                } else {
                    relative_path_str
                };

                match result {
                    Ok(()) => {
                        // Multi-test mode: minimal output with colored checkmark and dimmed path
                        let path_colored = if should_color() {
                            format!("{}{}{}", colors::GREEN, "✓ ", colors::RESET)
                                + &format!("{}{}{}", colors::DIM, display_path, colors::RESET)
                        } else {
                            format!("✓ {}", display_path)
                        };
                        println!("{}", path_colored);
                        passed += 1;
                    }
                    Err(_e) => {
                        // Multi-test mode: minimal output with colored X and dimmed path
                        // Error details (including panic messages) are suppressed in multi-test mode
                        // Panics are caught by the concurrent runner and converted to errors
                        let path_colored = if should_color() {
                            format!("{}{}{}", colors::RED, "✗ ", colors::RESET)
                                + &format!("{}{}{}", colors::DIM, display_path, colors::RESET)
                        } else {
                            format!("✗ {}", display_path)
                        };
                        println!("{}", path_colored);
                        failed += 1;
                        failed_tests.push(FailedTest {
                            path: spec.path.clone(),
                            line_number: spec.line_number,
                        });
                    }
                }
                reported_tests += 1;
                continue;
            }
        }

        // If we can't report the next test yet, wait for more replies
        if let Some(reply) = concurrent_runner.get() {
            match reply {
                concurrent::Reply::Done { jobid, result } => {
                    tests[jobid].state = TestState::Done(result);
                }
            }
        }
    }

    // Shutdown threads
    concurrent_runner.shutdown();
    concurrent_runner.join();

    let elapsed = start_time.elapsed();
    print_failed_tests_summary(&failed_tests, &filetests_dir, !show_full_output);
    println!("\n{}", format_results_summary(passed, failed, elapsed));

    if failed > 0 {
        anyhow::bail!("{} test file(s) failed", failed);
    }

    Ok(())
}

/// Format results summary with colors and timing.
/// Returns a single line with passed and failed counts, colored appropriately.
fn format_results_summary(passed: usize, failed: usize, elapsed: std::time::Duration) -> String {
    let seconds = elapsed.as_secs_f64();
    let time_str = if seconds < 1.0 {
        format!("{:.0}ms", elapsed.as_millis())
    } else if seconds < 60.0 {
        format!("{:.2}s", seconds)
    } else {
        let mins = elapsed.as_secs() / 60;
        let remaining_secs = elapsed.as_secs_f64() - (mins * 60) as f64;
        format!("{}m {:.2}s", mins, remaining_secs)
    };

    if should_color() {
        let passed_part = format!("{}{} passed{}", colors::GREEN, passed, colors::RESET);
        if failed > 0 {
            let failed_part = format!("{}{} failed{}", colors::RED, failed, colors::RESET);
            format!("{}, {} in {}", passed_part, failed_part, time_str)
        } else {
            format!("{} in {}", passed_part, time_str)
        }
    } else {
        if failed > 0 {
            format!("{} passed, {} failed in {}", passed, failed, time_str)
        } else {
            format!("{} passed in {}", passed, time_str)
        }
    }
}

/// Print summary of failed tests.
fn print_failed_tests_summary(
    failed_tests: &[FailedTest],
    filetests_dir: &Path,
    show_summary: bool,
) {
    if show_summary && !failed_tests.is_empty() {
        // Print title with count and bold styling
        let title = if should_color() {
            format!(
                "{}{}{} Failed tests{}",
                colors::RED,
                colors::BOLD,
                failed_tests.len(),
                colors::RESET
            )
        } else {
            format!("{} Failed tests", failed_tests.len())
        };
        println!("\n{}", title);

        // Print explanation message
        let explanation = if should_color() {
            format!(
                "{}Run these commands to see test failure details{}\n",
                colors::DIM,
                colors::RESET
            )
        } else {
            "Run these commands to see test failure details\n".to_string()
        };
        println!("{}", explanation);

        for failed_test in failed_tests {
            // Compute relative path from filetests_dir
            let relative_path = failed_test
                .path
                .strip_prefix(filetests_dir)
                .unwrap_or(&failed_test.path)
                .to_string_lossy()
                .to_string();

            let test_path = if let Some(line) = failed_test.line_number {
                format!("{}:{}", relative_path, line)
            } else {
                relative_path.clone()
            };

            // Color the command prefix normally, path in dim color
            if should_color() {
                println!(
                    "scripts/glsl-filetests.sh {}{}{}",
                    colors::DIM,
                    test_path,
                    colors::RESET
                );
            } else {
                println!("scripts/glsl-filetests.sh {}", test_path);
            }
        }
    }
}
