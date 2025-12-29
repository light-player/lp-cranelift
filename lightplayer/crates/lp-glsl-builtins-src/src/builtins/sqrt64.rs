//! 64-bit integer square root.
//!
//! This implementation delegates to Rust's standard library `u64::isqrt()` function.
//! Since we compile to CLIF using rustc-codegen-cranelift, Rust's implementation
//! will be compiled to CLIF and extracted as part of our builtin.

/// Compute integer square root of u64.
///
/// Returns the largest integer `s` such that `s * s <= n`.
/// This delegates to Rust's standard library implementation.
#[no_mangle]
pub extern "C" fn __lp_sqrt_u64(n: u64) -> u64 {
    n.isqrt()
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    extern crate std;
    use super::__lp_sqrt_u64;

    #[test]
    fn test_basic() {
        assert_eq!(__lp_sqrt_u64(0), 0);
        assert_eq!(__lp_sqrt_u64(1), 1);
        assert_eq!(__lp_sqrt_u64(4), 2);
        assert_eq!(__lp_sqrt_u64(9), 3);
        assert_eq!(__lp_sqrt_u64(16), 4);
        assert_eq!(__lp_sqrt_u64(25), 5);
        assert_eq!(__lp_sqrt_u64(100), 10);
    }

    #[test]
    fn test_range() {
        assert_eq!(__lp_sqrt_u64(0), 0);
        assert_eq!(__lp_sqrt_u64(1), 1);
        assert_eq!(__lp_sqrt_u64(2), 1);
        assert_eq!(__lp_sqrt_u64(3), 1);
        assert_eq!(__lp_sqrt_u64(4), 2);
        assert_eq!(__lp_sqrt_u64(9), 3);
        assert_eq!(__lp_sqrt_u64(16), 4);
        assert_eq!(__lp_sqrt_u64(25), 5);
        assert_eq!(__lp_sqrt_u64(100), 10);
    }

    #[test]
    fn test_1000() {
        // Test sqrt(1000) - should be 31 (since 31^2 = 961 <= 1000 < 32^2 = 1024)
        let result = __lp_sqrt_u64(1000);
        std::println!("sqrt(1000) = {}", result);
        assert_eq!(result, 31);
        assert!(result * result <= 1000);
        assert!((result + 1) * (result + 1) > 1000);
    }
}
