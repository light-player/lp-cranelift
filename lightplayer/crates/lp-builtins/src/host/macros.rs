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
/// This macro expands to `__host_debug(core::format_args!(...))`.
/// The underlying function is linked differently depending on context:
/// - Emulator: Implemented in `lp-builtins-app` (syscall-based)
/// - Tests: Implemented in `lp-builtins` with `std` (gated by feature flag)
/// - JIT: Implemented in `lp-glsl` (delegates to `lp-glsl::debug!`)
#[macro_export]
macro_rules! host_debug {
    ($($arg:tt)*) => {
        $crate::host::__host_debug(core::format_args!($($arg)*));
    };
}

/// Println macro for host functions.
///
/// Usage:
/// ```ignore
/// lp_builtins::host_println!("message: {}", value);
/// ```
///
/// This macro expands to `__host_println(core::format_args!(...))`.
/// The underlying function is linked differently depending on context:
/// - Emulator: Implemented in `lp-builtins-app` (syscall-based)
/// - Tests: Implemented in `lp-builtins` with `std` (gated by feature flag)
/// - JIT: Implemented in `lp-glsl` (delegates to `std::println!`)
#[macro_export]
macro_rules! host_println {
    () => {
        $crate::host::__host_println(core::format_args!("\n"));
    };
    ($($arg:tt)*) => {
        $crate::host::__host_println(core::format_args!($($arg)*));
        $crate::host::__host_println(core::format_args!("\n"));
    };
}

