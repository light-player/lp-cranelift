//! Transformation passes for Cranelift IR
//!
//! This module contains various transformation passes that can be applied
//! to Cranelift IR after initial code generation.

pub mod fixed32;

pub use fixed32::FixedPointFormat;
