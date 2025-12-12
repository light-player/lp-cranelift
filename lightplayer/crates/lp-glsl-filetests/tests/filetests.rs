//! Test discovery and execution for GLSL filetests.
//!
//! This module discovers all `.glsl` files in the `filetests/` directory and runs them,
//! providing detailed output about which tests pass or fail.

use anyhow::Result;
use lp_glsl_filetests::{filetest, run_filetest};
use std::path::PathBuf;
use walkdir::WalkDir;

/// ANSI color codes for terminal output (matching Rust's test output style)
mod colors {
    pub const GREEN: &str = "\x1b[32m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
}

/// Check if colors should be enabled
/// Respects NO_COLOR environment variable
/// Colors are enabled by default (cargo test will handle TTY detection)
fn should_color() -> bool {
    // Respect NO_COLOR environment variable (https://no-color.org/)
    std::env::var("NO_COLOR").is_err()
}

/// Print colored text if TTY, otherwise plain text
fn print_colored(text: &str, color: &str) {
    if should_color() {
        print!("{}{}{}", color, text, colors::RESET);
    } else {
        print!("{}", text);
    }
}

/// Print colored text with newline if TTY, otherwise plain text
fn println_colored(text: &str, color: &str) {
    if should_color() {
        println!("{}{}{}", color, text, colors::RESET);
    } else {
        println!("{}", text);
    }
}

/// Generate individual test functions for each .glsl file at runtime.
/// This allows `cargo test` to show each file as a separate test.
#[test]
fn filetests() -> Result<()> {
    let filetests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");

    let mut test_files = Vec::new();
    for entry in WalkDir::new(&filetests_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("glsl") {
            test_files.push(path.to_path_buf());
        }
    }

    // Sort for deterministic output
    test_files.sort();

    let mut passed = 0;
    let mut failed = 0;

    println!("\nRunning {} test file(s)...\n", test_files.len());

    for path in &test_files {
        let relative_path = path
            .strip_prefix(&filetests_dir)
            .unwrap_or(path)
            .to_string_lossy();

        // Parse the file to get test count
        let test_file = match filetest::parse_test_file(path) {
            Ok(tf) => tf,
            Err(e) => {
                print!("test {} ... ", relative_path);
                println_colored("FAILED (parse error)", colors::RED);
                if should_color() {
                    println!("  {}Error:{} {:#}", colors::RED, colors::RESET, e);
                } else {
                    println!("  Error: {:#}", e);
                }
                failed += 1;
                continue;
            }
        };

        let test_count = test_file.run_directives.len();
        let test_label = if test_count > 0 {
            format!(
                "{} ({} test case{})",
                relative_path,
                test_count,
                if test_count == 1 { "" } else { "s" }
            )
        } else {
            relative_path.to_string()
        };

        print!("test {} ... ", test_label);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        match run_filetest(path) {
            Ok(()) => {
                println_colored("ok", colors::GREEN);
                passed += 1;
            }
            Err(e) => {
                println_colored("FAILED", colors::RED);
                // Format error output with better readability
                let error_str = format!("{:#}", e);

                // Process the error message line by line
                let mut in_code_block = false;
                for line in error_str.lines() {
                    // Detect code block sections (lines with "|")
                    if line.contains(" | ") {
                        if !in_code_block {
                            in_code_block = true;
                            // Add a blank line before code block
                            println!();
                        }
                        println!("  {}", line);
                    } else {
                        if in_code_block {
                            in_code_block = false;
                            // Add a blank line after code block
                            println!();
                        }
                        // Regular error text - indent appropriately
                        if line.trim().is_empty() {
                            println!();
                        } else if line.starts_with("  ") {
                            // Already indented (from anyhow formatting)
                            println!("{}", line);
                        } else {
                            println!("  {}", line);
                        }
                    }
                }
                failed += 1;
            }
        }
    }

    print!("\ntest result: ");
    if failed == 0 {
        print_colored("ok", colors::GREEN);
        println!(". {} passed; {} failed; 0 ignored", passed, failed);
    } else {
        print_colored("FAILED", colors::RED);
        println!(". {} passed; {} failed; 0 ignored", passed, failed);
    }

    if failed > 0 {
        anyhow::bail!("{} test file(s) failed", failed);
    }

    Ok(())
}
