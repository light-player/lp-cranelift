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

// Public modules
pub mod codegen;
pub mod error;
pub mod frontend;
#[cfg(feature = "intrinsic-math")]
pub mod intrinsics;
pub mod semantic;
pub mod transform;

// Private modules
mod backend;
mod compile;
mod compiler;
mod ir;
mod ir_utils;

// Re-exports
pub use backend::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode};
#[cfg(feature = "emulator")]
pub use backend::GlslEmulatorModule;
pub use backend::GlslJitModule;
pub use compiler::{Backend, CompiledShader, CompilationPipeline, ParseResult, SemanticResult, TransformationPass};
pub use error::{ErrorCode, GlslError, SourceLocation};
pub use ir::ClifModule;
pub use transform::FixedPointFormat;

// Public API functions
pub use compile::glsl_jit;

#[cfg(feature = "emulator")]
pub use compile::glsl_emu_riscv32;
