//! GLSL fragment shader compiler using Cranelift JIT.
//!
//! Phase 1: Basic architecture with int/bool support only.

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod frontend;
pub mod semantic;
pub mod codegen;
#[cfg(feature = "std")]
pub mod jit;
pub mod compiler;

// Testing module available for dev-dependencies (integration tests)
#[cfg(any(test, feature = "std"))]
pub mod testing;

pub use compiler::Compiler;
#[cfg(feature = "std")]
pub use jit::JIT;

