//! Top-level lib.rs for `cranelift_jit`.
//!
//! There is an [example project](https://github.com/bytecodealliance/cranelift-jit-demo/)
//! which shows how to use some of the features of `cranelift_jit`.
//!
//! ## no_std Support
//!
//! This crate supports `no_std` environments when built with `default-features = false`
//! and `features = ["core"]`. In `no_std` mode:
//!
//! - You must provide a custom [`JITMemoryProvider`] implementation
//! - All external symbols must be registered via [`JITBuilder::symbol`]
//! - System memory providers ([`SystemMemoryProvider`], [`ArenaMemoryProvider`]) are not available

#![deny(missing_docs, unreachable_pub)]
#![expect(unsafe_op_in_unsafe_fn, reason = "crate isn't migrated yet")]
#![no_std]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc as std;
#[cfg(feature = "std")]
#[macro_use]
extern crate std;

mod backend;
mod compiled_blob;
mod memory;

pub use crate::backend::{JITBuilder, JITModule};
pub use crate::memory::{BranchProtection, JITMemoryProvider};

#[cfg(feature = "system-memory")]
pub use crate::memory::{ArenaMemoryProvider, SystemMemoryProvider};

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
