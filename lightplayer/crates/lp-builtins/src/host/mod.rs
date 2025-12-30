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
/// The function takes a pointer to a formatted string and its length.
/// Parameters: (ptr: *const u8, len: usize)
#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub extern "C" fn __host_debug(_ptr: *const u8, _len: usize) {
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
/// The function takes a pointer to a formatted string and its length.
/// Parameters: (ptr: *const u8, len: usize)
#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub extern "C" fn __host_println(_ptr: *const u8, _len: usize) {
    // Default implementation: no-op (will be linked to actual implementation)
    // This is only used when the test feature is disabled and no other
    // implementation is linked in.
}

#[cfg(feature = "test")]
mod test;

#[cfg(feature = "test")]
pub use test::{__host_debug, __host_println};

#[cfg(test)]
#[cfg(feature = "test")]
mod tests;

