//! Filetest infrastructure for lp-glsl

pub mod execution;
pub mod file_update;
pub mod filetest;
pub mod test_compile;
pub mod test_error;
pub mod test_run;
pub mod test_riscv32_fixed32;

pub use filetest::{TestTarget, run_filetest};
