//! GLSL fragment shader compiler using Cranelift JIT.
//!
//! Phase 1: Basic architecture with int/bool support only.

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

pub mod error;
pub mod frontend;
pub mod semantic;
pub mod codegen;
pub mod transform;
#[cfg(feature = "std")]
pub mod jit;
pub mod compiler;

pub use compiler::Compiler;
pub use error::{ErrorCode, GlslError, SourceLocation};
#[cfg(feature = "std")]
pub use jit::JIT;
pub use transform::FixedPointFormat;

