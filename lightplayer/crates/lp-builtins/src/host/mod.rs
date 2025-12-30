//! Host functions for cross-context communication.
//!
//! This module provides functions like `debug!` and `println!` that work
//! differently depending on execution context:
//! - Emulator: Functions defined in `lp-builtins-app` (syscall-based)
//! - Tests: Functions defined here using `std` (gated by feature flag)
//! - JIT: Functions registered by `GlJitModule` (delegate to `lp-glsl` macros)

mod registry;
mod macros;

pub use registry::HostId;

// Macros are exported at crate root via #[macro_export]
// Users should use: lp_builtins::host_debug! and lp_builtins::host_println!
// Or we can re-export them here if needed

/// Debug function declaration.
///
/// This function is implemented differently depending on context:
/// - Emulator: Implemented in `lp-builtins-app` (syscall-based)
/// - Tests: Implemented here with `std` (gated by feature flag)
/// - JIT: Implemented in `lp-glsl` (delegates to `lp-glsl::debug!`)
///
/// The function takes `fmt::Arguments` which is created by the `host::debug!` macro
/// using `core::format_args!`.
///
/// Note: This is not `extern "C"` because `fmt::Arguments` cannot be passed via C ABI.
/// The function is linked at link time, not called via FFI.
#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub fn __host_debug(_args: core::fmt::Arguments) {
    // Default implementation: no-op (will be linked to actual implementation)
    // This is only used when the test feature is disabled and no other
    // implementation is linked in.
}

/// Println function declaration.
///
/// This function is implemented differently depending on context:
/// - Emulator: Implemented in `lp-builtins-app` (syscall-based)
/// - Tests: Implemented here with `std` (gated by feature flag)
/// - JIT: Implemented in `lp-glsl` (delegates to `std::println!`)
///
/// The function takes `fmt::Arguments` which is created by the `host::println!` macro
/// using `core::format_args!`.
///
/// Note: This is not `extern "C"` because `fmt::Arguments` cannot be passed via C ABI.
/// The function is linked at link time, not called via FFI.
#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub fn __host_println(_args: core::fmt::Arguments) {
    // Default implementation: no-op (will be linked to actual implementation)
    // This is only used when the test feature is disabled and no other
    // implementation is linked in.
}

#[cfg(feature = "test")]
mod test;

#[cfg(feature = "test")]
pub use test::{__host_debug, __host_println};

