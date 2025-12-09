//! Utilities for calling and wrapping JIT-compiled functions.
//!
//! This crate provides abstractions for:
//! - Calling StructReturn functions with correct calling conventions
//! - Wrapping StructReturn functions into Rust-friendly closures
//! - Handling platform-specific ABI requirements

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec, string::String};

pub mod call;
pub mod wrapper;
pub mod error;

pub use call::call_structreturn;
pub use wrapper::{StructReturnWrapper, wrap_structreturn_function};
pub use error::JitCallError;

