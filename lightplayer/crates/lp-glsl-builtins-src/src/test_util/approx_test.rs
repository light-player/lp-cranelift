//! Approximate equality testing utilities for fixed32 functions.

#[cfg(test)]
extern crate std;

extern crate alloc;
use alloc::{format, string::String, vec::Vec};

/// Test a fixed32 function with approximate equality checks.
///
/// # Arguments
/// * `function_name` - Name of the function being tested (for error messages)
/// * `test_cases` - Vector of (input, expected, tolerance) tuples
///   - input: f32 value (will be converted to fixed32)
///   - expected: f32 expected result
///   - tolerance: f32 tolerance for approximate comparison
/// * `func` - Closure that takes i32 (fixed32) and returns i32 (fixed32)
///
/// # Example
/// ```
/// test_fn_fx32(
///     "fixed32_sqrt",
///     vec![
///         (4.0, 2.0, 0.1),
///         (16.0, 4.0, 0.1),
///     ],
///     |i| fixed32_sqrt(i)
/// );
/// ```
pub fn test_fn_fx32<F>(function_name: &str, test_cases: Vec<(f32, f32, f32)>, func: F)
where
    F: Fn(i32) -> i32,
{
    const SCALE: f32 = 65536.0;

    let mut passed = 0;
    let mut failed = Vec::new();

    for (input_f, expected_f, tolerance) in test_cases {
        let input_fixed = (input_f * SCALE) as i32;
        let result_fixed = func(input_fixed);
        let result_f = result_fixed as f32 / SCALE;
        let diff = (result_f - expected_f).abs();

        if diff <= tolerance {
            passed += 1;
            #[cfg(test)]
            std::println!("✓ {}({}) ~= {}±{}; actual {}", function_name, input_f, expected_f, tolerance, result_f);
        } else {
            failed.push((input_f, expected_f, tolerance, result_f, diff));
            #[cfg(test)]
            std::println!("✗ {}({}) ~= {}±{}; actual {} (diff: {})", function_name, input_f, expected_f, tolerance, result_f, diff);
        }
    }

    if !failed.is_empty() {
        #[cfg(test)]
        std::println!("\nFailed tests:");
        for (input_f, expected_f, tolerance, result_f, diff) in &failed {
            #[cfg(test)]
            std::println!("  ✗ {}({}) ~= {}±{}; actual {} (diff: {})", function_name, input_f, expected_f, tolerance, result_f, diff);
        }
        panic!("{} test(s) failed for '{}' ({} passed)", failed.len(), function_name, passed);
    }

    #[cfg(test)]
    std::println!("All {} test(s) passed for '{}'", passed, function_name);
}

