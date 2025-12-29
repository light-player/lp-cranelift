//! Test expectation data structures.

extern crate alloc;
use crate::test_util::number::TestNum;
use alloc::{string::String, vec, vec::Vec};

/// Comparison operator for test expectations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonOp {
    /// Exact equality: `==`
    Exact,
    /// Approximate equality: `~=` with tolerance
    Approx {
        /// Tolerance value (default: 0.000001 for float32)
        tolerance: f32,
    },
}

impl ComparisonOp {
    /// Default tolerance for approximate comparisons (0.000001).
    pub const DEFAULT_TOLERANCE: f32 = 0.000001;

    /// Create an approximate comparison with default tolerance.
    pub fn approx_default() -> Self {
        Self::Approx {
            tolerance: Self::DEFAULT_TOLERANCE,
        }
    }

    /// Create an approximate comparison with custom tolerance.
    pub fn approx(tolerance: f32) -> Self {
        Self::Approx { tolerance }
    }
}

/// A parsed run directive from a `// run:` line.
#[derive(Debug, Clone)]
pub struct RunDirective {
    /// Function name (without `%` prefix)
    pub function_name: String,
    /// Function arguments (parsed numbers)
    pub arguments: Vec<TestNum>,
    /// Format of each argument (parallel to arguments)
    pub arg_formats: Vec<crate::test_util::number::NumFormat>,
    /// Expected result (parsed number)
    pub expected: TestNum,
    /// Format of expected result
    pub expected_format: crate::test_util::number::NumFormat,
    /// Comparison operator
    pub comparison: ComparisonOp,
    /// Line number in source file
    pub line_number: usize,
}

impl RunDirective {
    /// Check if this directive matches the given function name.
    pub fn matches_function(&self, function_name: &str) -> bool {
        self.function_name == function_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::number::{NumFormat, TestNum};

    #[test]
    fn test_comparison_op_default() {
        let op = ComparisonOp::approx_default();
        assert!(matches!(op, ComparisonOp::Approx { tolerance: 0.000001 }));
    }

    #[test]
    fn test_comparison_op_custom() {
        let op = ComparisonOp::approx(0.001);
        assert!(matches!(op, ComparisonOp::Approx { tolerance: 0.001 }));
    }

    #[test]
    fn test_run_directive_matching() {
        extern crate alloc;
        use alloc::{string::ToString, vec};
        
        let directive = RunDirective {
            function_name: "test_func".to_string(),
            arguments: vec![],
            arg_formats: vec![],
            expected: TestNum::I32(0),
            expected_format: NumFormat::Decimal,
            comparison: ComparisonOp::Exact,
            line_number: 1,
        };

        assert!(directive.matches_function("test_func"));
        assert!(!directive.matches_function("other_func"));
    }
}

