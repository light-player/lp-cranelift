//! File-based tests for the Cranelift RISC-V32 backend.
//!
//! These tests compile CLIF (Cranelift IR) to RISC-V32 machine code and verify
//! the output using filecheck patterns and emulator execution.

pub mod filecheck;
pub mod compile;
