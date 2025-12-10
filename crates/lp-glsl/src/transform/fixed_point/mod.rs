//! Float-to-fixed-point transformation pass.
//!
//! This module converts floating point operations to fixed-point arithmetic
//! at the Cranelift IR level. All F32 types and operations are converted to
//! fixed-point representation using either I32 (16.16 format) or I64 (32.32 format).

mod converters;
mod rewrite;
mod transform;
mod types;

#[cfg(test)]
mod rewrite_test;

pub use transform::convert_floats_to_fixed;
pub use types::{FixedPointFormat, float_to_fixed16x16, float_to_fixed32x32};
