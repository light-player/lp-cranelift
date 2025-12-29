//! Function to run tests from `// run:` directives.

extern crate alloc;

use crate::test_util::expectations::RunDirective;
use crate::test_util::number::{NumType, TestNum};
use crate::test_util::parser::parse_run_directives;
use alloc::{format, string::String, vec, vec::Vec};

/// Run tests for a function with i32 argument and return value.
///
/// Usage:
/// ```rust
/// run_runtests_i32(
///     include_str!(file!()),
///     "function_name",
///     |x| function(x),
/// );
/// ```
pub fn run_runtests_i32<F>(source: &str, function_name: &str, closure: F)
where
    F: Fn(i32) -> i32,
{
    run_runtests(
        source,
        function_name,
        NumType::I32,
        vec![NumType::I32],
        |args| {
            // Extract i32 from TestNum
            let arg = match args[0] {
                TestNum::I32(v) => v,
                _ => panic!("expected i32 argument"),
            };
            let result = closure(arg);
            TestNum::I32(result)
        },
    )
}

/// Generic run_runtests function.
///
/// Takes:
/// - `source`: Source file content with `// run:` directives
/// - `function_name`: Name of function to test
/// - `return_type`: Expected return type
/// - `arg_types`: Expected argument types (in order)
/// - `closure`: Function that takes Vec<TestNum> and returns TestNum
pub fn run_runtests<F>(
    source: &str,
    function_name: &str,
    return_type: NumType,
    arg_types: Vec<NumType>,
    closure: F,
) where
    F: Fn(Vec<TestNum>) -> TestNum,
{
    // Parse run directives
    let directives = parse_run_directives(source)
        .unwrap_or_else(|e| panic!("failed to parse run directives: {}", e));

    // Filter directives for this function
    let relevant_directives: Vec<&RunDirective> = directives
        .iter()
        .filter(|d| d.matches_function(function_name))
        .collect();

    let num_directives = relevant_directives.len();
    if num_directives == 0 {
        panic!("no run directives found for function '{}'", function_name);
    }

    // Collect failures (only for test execution failures, not type errors)
    let mut failures = Vec::new();

    for directive in relevant_directives {
        // Type check: verify argument count
        if directive.arguments.len() != arg_types.len() {
            panic!(
                "Line {}: type error - expected {} argument(s), got {}",
                directive.line_number,
                arg_types.len(),
                directive.arguments.len()
            );
        }

        // Type check: verify argument types match
        for (i, (arg, expected_type)) in
            directive.arguments.iter().zip(arg_types.iter()).enumerate()
        {
            let matches = match (arg, expected_type) {
                (TestNum::I32(_), NumType::I32) => true,
                (TestNum::U32(_), NumType::U32) => true,
                (TestNum::F32(_), NumType::F32) => true,
                _ => false,
            };
            if !matches {
                panic!(
                    "Line {}: type error - argument {} type mismatch (expected {:?}, got {:?})",
                    directive.line_number,
                    i,
                    expected_type,
                    match arg {
                        TestNum::I32(_) => "i32",
                        TestNum::U32(_) => "u32",
                        TestNum::F32(_) => "f32",
                    }
                );
            }
        }

        // Execute function
        let result = closure(directive.arguments.clone());

        // Type check: verify return type matches
        let return_matches = match (&result, &return_type) {
            (TestNum::I32(_), NumType::I32) => true,
            (TestNum::U32(_), NumType::U32) => true,
            (TestNum::F32(_), NumType::F32) => true,
            _ => false,
        };
        if !return_matches {
            panic!(
                "Line {}: type error - return type mismatch (expected {:?}, got {:?})",
                directive.line_number,
                return_type,
                match result {
                    TestNum::I32(_) => "i32",
                    TestNum::U32(_) => "u32",
                    TestNum::F32(_) => "f32",
                }
            );
        }

        // Compare results
        let passed = match (&result, &directive.expected, &directive.comparison) {
            (
                TestNum::I32(r),
                TestNum::I32(e),
                crate::test_util::expectations::ComparisonOp::Exact,
            ) => r == e,
            (
                TestNum::I32(r),
                TestNum::I32(e),
                crate::test_util::expectations::ComparisonOp::Approx { tolerance },
            ) => {
                // Convert fixed32 to float for comparison
                let r_f = *r as f32 / 65536.0;
                let e_f = *e as f32 / 65536.0;
                (r_f - e_f).abs() <= *tolerance
            }
            (
                TestNum::U32(r),
                TestNum::U32(e),
                crate::test_util::expectations::ComparisonOp::Exact,
            ) => r == e,
            (
                TestNum::F32(r),
                TestNum::F32(e),
                crate::test_util::expectations::ComparisonOp::Exact,
            ) => r == e,
            (
                TestNum::F32(r),
                TestNum::F32(e),
                crate::test_util::expectations::ComparisonOp::Approx { tolerance },
            ) => (r - e).abs() <= *tolerance,
            _ => {
                // Type mismatch or unsupported comparison
                failures.push(format!(
                    "Line {}: cannot compare {:?} with {:?}",
                    directive.line_number, result, directive.expected
                ));
                continue;
            }
        };

        if !passed {
            let diff_msg = match (&result, &directive.expected, &directive.comparison) {
                (
                    TestNum::I32(r),
                    TestNum::I32(e),
                    crate::test_util::expectations::ComparisonOp::Approx { tolerance },
                ) => {
                    let r_f = *r as f32 / 65536.0;
                    let e_f = *e as f32 / 65536.0;
                    format!(" (diff: {}, tolerance: {})", (r_f - e_f).abs(), tolerance)
                }
                (
                    TestNum::F32(r),
                    TestNum::F32(e),
                    crate::test_util::expectations::ComparisonOp::Approx { tolerance },
                ) => {
                    format!(" (diff: {}, tolerance: {})", (r - e).abs(), tolerance)
                }
                _ => String::new(),
            };
            failures.push(format!(
                "Line {}: expected {:?}, got {:?}{}",
                directive.line_number, directive.expected, result, diff_msg
            ));
        }
    }

    // Report failures
    if !failures.is_empty() {
        let mut error_msg = format!(
            "{} test(s) failed for '{}':\n",
            failures.len(),
            function_name
        );
        for failure in failures {
            error_msg.push_str(&format!("  - {}\n", failure));
        }
        panic!("{}", error_msg);
    }
}
