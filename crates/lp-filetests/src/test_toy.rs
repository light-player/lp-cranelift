//! The `toy` subtest - toy language execution tests
//!
//! Tests that verify toy language programs compile and execute correctly
//! using the new SSABuilder API, producing expected output.

use lpc_toy_lang::{execute_function, Translator};

use crate::filecheck::match_filecheck;

/// Run tests from toy language test files
#[allow(dead_code)]
pub fn run_tests_from_file(content: &str) {
    let test_cases = parse_toy_test_file(content);

    assert!(!test_cases.is_empty(), "No test cases found in test file");

    for case in test_cases {
        run_toy_test(&case.toy_code, &case.args, &case.expected_output);
    }
}

/// A test case for toy language tests
#[derive(Debug, Clone)]
struct ToyTestCase {
    /// Toy language source code
    toy_code: String,
    /// Function arguments (as comma-separated integers)
    args: Vec<i32>,
    /// Expected output (filecheck directives or plain text)
    expected_output: String,
}

/// Parse toy language test file into test cases
fn parse_toy_test_file(content: &str) -> Vec<ToyTestCase> {
    let lines: Vec<&str> = content.lines().collect();
    let mut test_cases = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for test command
        if line.starts_with("test toy") {
            i += 1;
            // Skip blank lines
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Parse toy code block (until we hit "args:" or end)
            let mut toy_code = String::new();
            while i < lines.len() {
                let line = lines[i];
                if line.trim().starts_with("args:") {
                    break;
                }
                toy_code.push_str(line);
                toy_code.push('\n');
                i += 1;
            }

            // Parse all test cases for this function
            while i < lines.len() {
                // Check if we've hit the next test command
                if lines[i].trim().starts_with("test toy") {
                    break;
                }

                // Parse args
                let mut args = Vec::new();
                if i < lines.len() && lines[i].trim().starts_with("args:") {
                    let args_line = lines[i].trim();
                    let args_str = args_line.strip_prefix("args:").unwrap_or("").trim();
                    if !args_str.is_empty() {
                        for arg_str in args_str.split(',') {
                            if let Ok(arg) = arg_str.trim().parse::<i32>() {
                                args.push(arg);
                            }
                        }
                    }
                    i += 1;
                }

                // Parse expected output (check: directive or following lines)
                let mut expected_output = String::new();
                while i < lines.len() {
                    let line = lines[i];
                    if line.trim().starts_with("test toy") || line.trim().starts_with("args:") {
                        break; // Next test case or next function
                    }
                    // Check for check: directives
                    if line.trim().starts_with("check:") {
                        expected_output.push_str(line);
                        expected_output.push('\n');
                    } else if !line.trim().is_empty() && !expected_output.is_empty() {
                        // Non-empty line after we've started collecting output
                        expected_output.push_str(line);
                        expected_output.push('\n');
                    }
                    i += 1;
                }

                if !args.is_empty() || !expected_output.trim().is_empty() {
                    test_cases.push(ToyTestCase {
                        toy_code: toy_code.trim().to_string(),
                        args,
                        expected_output: expected_output.trim().to_string(),
                    });
                }
            }
        } else {
            i += 1;
        }
    }

    test_cases
}

/// Run a single toy language test
#[allow(dead_code)]
fn run_toy_test(toy_code: &str, args: &[i32], expected_output: &str) {
    // Compile toy language program
    let mut translator = Translator::new();
    let func = match translator.compile(toy_code) {
        Ok(f) => f,
        Err(e) => {
            panic!(
                "Failed to compile toy language program: {}\n\nCode:\n{}",
                e, toy_code
            );
        }
    };

    // Format LPIR output for validation
    let lpir_output = format!("{}", func);

    // Parse expected output to separate execution results from LPIR checks
    let (execution_checks, lpir_checks) = parse_checks(expected_output);

    // Validate LPIR output if checks are provided
    if !lpir_checks.trim().is_empty() {
        if let Err(e) = match_filecheck(&lpir_output, &lpir_checks) {
            panic!(
                "Toy language LPIR validation failed (filecheck):\n{}\n\nExpected \
                 LPIR:\n{}\n\nActual LPIR:\n{}\n\nCode:\n{}",
                e, lpir_checks, lpir_output, toy_code
            );
        }
    }

    // Execute the function
    let result = match execute_function(func, args) {
        Ok(r) => r,
        Err(e) => {
            panic!(
                "Failed to execute toy language program: {}\n\nCode:\n{}",
                e, toy_code
            );
        }
    };

    // Format output
    let actual_output = format!("result: {}", result);

    // Verify execution expectations using filecheck or exact match
    if !execution_checks.trim().is_empty() {
        // Try filecheck first
        if execution_checks.contains("check:") {
            if let Err(e) = match_filecheck(&actual_output, &execution_checks) {
                panic!(
                    "Toy language test failed \
                     (filecheck):\n{}\n\nExpected:\n{}\n\nActual:\n{}\n\nCode:\n{}",
                    e, execution_checks, actual_output, toy_code
                );
            }
        } else {
            // Simple text matching
            let expected_normalized = execution_checks.trim();
            let actual_normalized = actual_output.trim();
            if !actual_normalized.contains(expected_normalized)
                && expected_normalized != actual_normalized
            {
                panic!(
                    "Toy language test failed:\n\nExpected:\n{}\n\nActual:\n{}\n\nCode:\n{}",
                    execution_checks, actual_output, toy_code
                );
            }
        }
    }
}

/// Parse check directives, separating execution checks from LPIR checks
fn parse_checks(expected_text: &str) -> (String, String) {
    let mut execution_checks = String::new();
    let mut lpir_checks = String::new();
    let mut in_lpir_section = false;

    for line in expected_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("check:") {
            let rest = trimmed[6..].trim();
            if rest.starts_with("lpir:") || rest.starts_with("# LPIR") {
                in_lpir_section = true;
                lpir_checks.push_str(line);
                lpir_checks.push('\n');
            } else {
                in_lpir_section = false;
                execution_checks.push_str(line);
                execution_checks.push('\n');
            }
        } else if in_lpir_section && !trimmed.is_empty() {
            lpir_checks.push_str(line);
            lpir_checks.push('\n');
        } else if !in_lpir_section && !trimmed.is_empty() {
            execution_checks.push_str(line);
            execution_checks.push('\n');
        }
    }

    (
        execution_checks.trim().to_string(),
        lpir_checks.trim().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toy_basic() {
        let content = include_str!("../filetests/toy/basic.toy");
        run_tests_from_file(content);
    }

    #[test]
    fn test_toy_if_else() {
        let content = include_str!("../filetests/toy/if_else.toy");
        run_tests_from_file(content);
    }

    #[test]
    fn test_toy_while_loop() {
        let content = include_str!("../filetests/toy/while_loop.toy");
        run_tests_from_file(content);
    }
}
