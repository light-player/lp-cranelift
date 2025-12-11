//! Float-to-fixed-point transformation pass.
//!
//! This module converts floating point operations to fixed-point arithmetic
//! at the Cranelift IR level. All F32 types and operations are converted to
//! fixed-point representation using I32 (16.16 format).

mod converters;
mod module;
mod rewrite;
mod transform;
mod types;

#[cfg(test)]
mod rewrite_test;

pub use module::transform_module;
pub use rewrite::convert_signature;
pub use transform::convert_floats_to_fixed;
pub use types::{FixedPointFormat, float_to_fixed16x16};
