//! Macros for host functions.
//!
//! These macros expand to calls to the underlying host functions,
//! following the same pattern as `lp-builtins-app` `println!` macros.

/// Debug macro for host functions.
///
/// Usage:
/// ```ignore
/// lp_builtins::host_debug!("message: {}", value);
/// ```
///
/// This macro formats the string first, then calls `__host_debug(&str)`.
/// The underlying function is linked differently depending on context:
/// - Emulator: Implemented in `lp-builtins-app` (syscall-based)
/// - Tests: Implemented in `lp-builtins` with `std` (gated by feature flag)
/// - JIT: Implemented in `lp-glsl` (delegates to `lp-glsl::debug!`)
#[macro_export]
macro_rules! host_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(all(feature = "std", feature = "test"))]
            {
                // With std and test feature, use std::format! and call test implementation
                let formatted = std::format!($($arg)*);
                $crate::host::__host_debug(formatted.as_ptr(), formatted.len());
            }
            #[cfg(not(feature = "std"))]
            {
                // Without std, use core::format_args! and format into static buffer
                $crate::host::_debug_format(core::format_args!($($arg)*));
            }
            #[cfg(all(feature = "std", not(feature = "test")))]
            {
                // With std but not test (shouldn't happen in lp-builtins, but handle gracefully)
                // This would be for JIT context, but JIT should use lp-glsl macros instead
                let formatted = std::format!($($arg)*);
                unsafe extern "C" {
                    fn __host_debug(ptr: *const u8, len: usize);
                }
                unsafe {
                    __host_debug(formatted.as_ptr(), formatted.len());
                }
            }
        }
    };
}

/// Println macro for host functions.
///
/// Usage:
/// ```ignore
/// lp_builtins::host_println!("message: {}", value);
/// ```
///
/// This macro formats the string first, then calls `__host_println(&str)`.
/// The underlying function is linked differently depending on context:
/// - Emulator: Implemented in `lp-builtins-app` (syscall-based)
/// - Tests: Implemented in `lp-builtins` with `std` (gated by feature flag)
/// - JIT: Implemented in `lp-glsl` (delegates to `std::println!`)
#[macro_export]
macro_rules! host_println {
    () => {
        let newline = "\n";
        #[cfg(all(feature = "std", feature = "test"))]
        {
            $crate::host::__host_println(newline.as_ptr(), newline.len());
        }
        #[cfg(not(feature = "std"))]
        {
            $crate::host::_println_format(core::format_args!("{}", newline));
        }
        #[cfg(all(feature = "std", not(feature = "test")))]
        {
            unsafe extern "C" {
                fn __host_println(ptr: *const u8, len: usize);
            }
            unsafe {
                __host_println(newline.as_ptr(), newline.len());
            }
        }
    };
    ($($arg:tt)*) => {
        {
            #[cfg(all(feature = "std", feature = "test"))]
            {
                // With std and test feature, use std::format! and call test implementation
                let formatted = std::format!($($arg)*);
                $crate::host::__host_println(formatted.as_ptr(), formatted.len());
            }
            #[cfg(not(feature = "std"))]
            {
                // Without std, use core::format_args! and format into static buffer
                $crate::host::_println_format(core::format_args!($($arg)*));
            }
            #[cfg(all(feature = "std", not(feature = "test")))]
            {
                // With std but not test (shouldn't happen in lp-builtins, but handle gracefully)
                let formatted = std::format!($($arg)*);
                unsafe extern "C" {
                    fn __host_println(ptr: *const u8, len: usize);
                }
                unsafe {
                    __host_println(formatted.as_ptr(), formatted.len());
                }
            }
        }
    };
}
