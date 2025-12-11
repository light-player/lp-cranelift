//! Float-to-fixed-point transformation pass.
//!
//! This module converts floating point operations to fixed-point arithmetic
//! at the Cranelift IR level. All F32 types and operations are converted to
//! fixed-point representation using I32 (16.16 format).

mod blocks;
pub mod converters;
mod function;
mod instructions;
mod module;
mod signature;
mod types;

pub use function::rewrite_function;
pub use module::transform_module;
pub use signature::convert_signature;
pub use types::{FixedPointFormat, float_to_fixed16x16};
