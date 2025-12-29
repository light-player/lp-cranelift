//! Test utilities for builtin functions.
//!
//! This module provides infrastructure for parsing and executing test expectations
//! from `// run:` directives in source files.

pub mod approx_test;
pub mod expectations;
pub mod number;
pub mod parser;
pub mod run_runtests;
pub mod clif;

pub use approx_test::test_fn_fx32;
pub use expectations::{ComparisonOp, RunDirective};
pub use number::{NumFormat, NumType, TestNum};
pub use run_runtests::run_runtests_i32;

