//! Fixed-point 16.16 arithmetic builtins.
//!
//! Functions operate on i32 values representing fixed-point numbers
//! with 16 bits of fractional precision.

mod div;
mod mul;
mod sqrt;

pub use div::__lp_fixed32_div;
pub use mul::__lp_fixed32_mul;
pub use sqrt::__lp_fixed32_sqrt;

