mod sqrt_recip;

use sqrt_recip::fixed32_sqrt;

/// Square root for fixed16x16 format.
///
/// This is a wrapper function that will be extracted from CLIF.
/// The function signature matches the expected CLIF signature.
#[no_mangle]
pub extern "C" fn __lp_fixed32_sqrt_recip(input: i32) -> i32 {
    fixed32_sqrt(input)
}