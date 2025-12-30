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
            // Format the string first
            let formatted = alloc::format!($($arg)*);
            $crate::host::__host_debug(formatted.as_ptr(), formatted.len());
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
        $crate::host::__host_println(newline.as_ptr(), newline.len());
    };
    ($($arg:tt)*) => {
        {
            // Format the string first
            let formatted = alloc::format!($($arg)*);
            $crate::host::__host_println(formatted.as_ptr(), formatted.len());
        }
    };
}

