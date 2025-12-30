#![cfg_attr(not(feature = "std"), no_std)]

//! Light Player builtins library.
//!
//! This crate provides low-level builtin functions for the Light Player compiler.
//! Functions are exported with `#[no_mangle] pub extern "C"` for linking.

pub mod fixed32;
// mem module only used when baremetal feature is not enabled
// When baremetal is enabled, rlibc provides memcpy/memset/memcmp
#[cfg(not(feature = "baremetal"))]
pub mod mem;

// When baremetal feature is enabled, rlibc provides memcpy/memset/memcmp
// No need to re-export - rlibc exports them with #[no_mangle]

// Panic handler must be provided by the executable that uses this library
// This crate is only used as a dependency, never built standalone
