//! Filetest infrastructure for lp-glsl

pub mod filetest;
pub mod file_update;
pub mod test_compile;
pub mod test_run;
pub mod test_error;

pub use filetest::{run_filetest, TestTarget};

