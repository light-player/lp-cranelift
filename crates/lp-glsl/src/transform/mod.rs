//! Transformation passes for Cranelift IR
//!
//! This module contains various transformation passes that can be applied
//! to Cranelift IR after initial code generation.

pub mod fixed_point;

pub use fixed_point::{FixedPointFormat, TransformError, convert_floats_to_fixed};

