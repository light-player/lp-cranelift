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

pub mod codegen;
pub mod compiler;
pub mod error;
pub mod frontend;
#[cfg(feature = "intrinsic-math")]
pub mod intrinsics;
#[cfg(feature = "std")]
pub mod jit;
pub mod pipeline;
pub mod semantic;
pub mod transform;

pub use compiler::Compiler;
pub use error::{ErrorCode, GlslError, SourceLocation};
#[cfg(feature = "std")]
pub use jit::JIT;
pub use pipeline::{Backend, CompiledShader, ParseResult, SemanticResult, TransformationPass};
#[cfg(feature = "std")]
pub use pipeline::{CLIFBackend, JITBackend};
pub use transform::FixedPointFormat;
#[cfg(feature = "std")]
pub use transform::FixedPointTransformation;
