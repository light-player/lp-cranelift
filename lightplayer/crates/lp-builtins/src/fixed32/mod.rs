//! Fixed-point 16.16 arithmetic builtins.
//!
//! Functions operate on i32 values representing fixed-point numbers
//! with 16 bits of fractional precision.

mod div;
mod mul;
mod sin;
mod cos;
mod tan;
mod atan2;
mod atan;
mod asin;
mod acos;
mod sqrt;
mod exp;
mod log;
mod log2;
mod exp2;
mod sinh;
mod cosh;
mod tanh;
mod asinh;
mod acosh;
mod atanh;
mod pow;

#[cfg(test)]
mod test_helpers;

pub use div::__lp_fixed32_div;
pub use mul::__lp_fixed32_mul;
pub use sin::__lp_fixed32_sin;
pub use cos::__lp_fixed32_cos;
pub use tan::__lp_fixed32_tan;
pub use atan2::__lp_fixed32_atan2;
pub use atan::__lp_fixed32_atan;
pub use asin::__lp_fixed32_asin;
pub use acos::__lp_fixed32_acos;
pub use sqrt::__lp_fixed32_sqrt;
pub use exp::__lp_fixed32_exp;
pub use log::__lp_fixed32_log;
pub use log2::__lp_fixed32_log2;
pub use exp2::__lp_fixed32_exp2;
pub use sinh::__lp_fixed32_sinh;
pub use cosh::__lp_fixed32_cosh;
pub use tanh::__lp_fixed32_tanh;
pub use asinh::__lp_fixed32_asinh;
pub use acosh::__lp_fixed32_acosh;
pub use atanh::__lp_fixed32_atanh;
pub use pow::__lp_fixed32_pow;
