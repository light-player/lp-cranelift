//! Render test expectations to CLIF filetest format.

extern crate alloc;
use crate::test_util::expectations::RunDirective;
use crate::test_util::number::{NumFormat, TestNum};
use alloc::{format, string::String, vec::Vec};

/// Render run directives to CLIF filetest format.
///
/// Generates `; run:` lines that can be appended to a CLIF file.
/// Preserves original number formats when possible.
pub fn render_to_clif(directives: &[RunDirective], function_name: &str) -> String {
    let mut result = String::new();

    for directive in directives {
        if !directive.matches_function(function_name) {
            continue;
        }

        // Format arguments
        let args_str = directive
            .arguments
            .iter()
            .zip(directive.arg_formats.iter())
            .map(|(arg, format)| format_number_for_clif(*arg, *format))
            .collect::<Vec<_>>()
            .join(", ");

        // Format comparison operator
        let op = match directive.comparison {
            crate::test_util::expectations::ComparisonOp::Exact => "==",
            crate::test_util::expectations::ComparisonOp::Approx { .. } => "~=",
        };

        // Format expected value
        let expected_str = format_number_for_clif(directive.expected, directive.expected_format);

        // Format tolerance if present
        let tolerance_str = match directive.comparison {
            crate::test_util::expectations::ComparisonOp::Exact => String::new(),
            crate::test_util::expectations::ComparisonOp::Approx { tolerance } => {
                format!(" (tolerance: {})", tolerance)
            }
        };

        // Generate run directive
        result.push_str(&format!(
            "; run: %{}({}) {} {}{}\n",
            function_name, args_str, op, expected_str, tolerance_str
        ));
    }

    result
}

/// Format a parsed number for CLIF output, preserving original format when possible.
fn format_number_for_clif(num: TestNum, format: NumFormat) -> String {
    match (num, format) {
        (TestNum::I32(value), NumFormat::Hex) => format!("0x{:08x}", value as u32),
        (TestNum::I32(value), NumFormat::Fixed32) => {
            // Convert back to float representation for readability
            let float_val = value as f32 / 65536.0;
            format!("{:.1}fx32", float_val)
        }
        (TestNum::I32(value), NumFormat::Decimal | NumFormat::Float32) => format!("{}", value),
        (TestNum::U32(value), NumFormat::Hex) => format!("0x{:08x}", value),
        (TestNum::U32(value), NumFormat::Decimal | NumFormat::Fixed32 | NumFormat::Float32) => {
            format!("{}", value)
        }
        (TestNum::F32(value), NumFormat::Float32) => format!("{}f32", value),
        (TestNum::F32(value), _) => format!("{}", value),
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::test_util::expectations::{ComparisonOp, RunDirective};
    use crate::test_util::number::{NumFormat, TestNum};
    use alloc::{format, string::ToString, vec};

    #[test]
    fn test_render_exact() {
        let directives = vec![RunDirective {
            function_name: "test_func".to_string(),
            arguments: vec![TestNum::I32(0x00040000)],
            arg_formats: vec![NumFormat::Hex],
            expected: TestNum::I32(0x00020000),
            expected_format: NumFormat::Hex,
            comparison: ComparisonOp::Exact,
            line_number: 1,
        }];

        let clif = render_to_clif(&directives, "test_func");
        assert!(clif.contains("; run: %test_func(0x00040000) == 0x00020000"));
    }

    #[test]
    fn test_render_approximate() {
        let directives = vec![RunDirective {
            function_name: "test_func".to_string(),
            arguments: vec![TestNum::I32(65536)],
            arg_formats: vec![NumFormat::Fixed32],
            expected: TestNum::I32(65536),
            expected_format: NumFormat::Fixed32,
            comparison: ComparisonOp::Approx { tolerance: 0.001 },
            line_number: 1,
        }];

        let clif = render_to_clif(&directives, "test_func");
        assert!(clif.contains("~="));
        assert!(clif.contains("(tolerance: 0.001)"));
    }

    #[test]
    fn test_render_fixed32_preserves_format() {
        let directives = vec![RunDirective {
            function_name: "test_func".to_string(),
            arguments: vec![TestNum::I32(262144)], // 4.0 * 65536
            arg_formats: vec![NumFormat::Fixed32],
            expected: TestNum::I32(131072), // 2.0 * 65536
            expected_format: NumFormat::Fixed32,
            comparison: ComparisonOp::Exact,
            line_number: 1,
        }];

        let clif = render_to_clif(&directives, "test_func");
        assert!(clif.contains("4.0fx32"));
        assert!(clif.contains("2.0fx32"));
    }

    #[test]
    fn test_render_multiple_args() {
        let directives = vec![RunDirective {
            function_name: "test_func".to_string(),
            arguments: vec![TestNum::I32(1), TestNum::I32(2)],
            arg_formats: vec![NumFormat::Decimal, NumFormat::Decimal],
            expected: TestNum::I32(3),
            expected_format: NumFormat::Decimal,
            comparison: ComparisonOp::Exact,
            line_number: 1,
        }];

        let clif = render_to_clif(&directives, "test_func");
        assert!(clif.contains("test_func(1, 2)"));
    }

    #[test]
    fn test_render_filters_by_function_name() {
        let directives = vec![
            RunDirective {
                function_name: "func1".to_string(),
                arguments: vec![],
                arg_formats: vec![],
                expected: TestNum::I32(1),
                expected_format: NumFormat::Decimal,
                comparison: ComparisonOp::Exact,
                line_number: 1,
            },
            RunDirective {
                function_name: "func2".to_string(),
                arguments: vec![],
                arg_formats: vec![],
                expected: TestNum::I32(2),
                expected_format: NumFormat::Decimal,
                comparison: ComparisonOp::Exact,
                line_number: 2,
            },
        ];

        let clif = render_to_clif(&directives, "func1");
        assert!(clif.contains("func1"));
        assert!(!clif.contains("func2"));
    }
}

