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

pub mod clif_module;
pub mod codegen;
pub mod compile;
pub mod compiler;
#[cfg(feature = "emulator")]
pub mod executable_emu;
pub mod error;
pub mod executable;
pub mod frontend;
pub mod glsl_compiler;
#[cfg(feature = "intrinsic-math")]
pub mod intrinsics;
#[cfg(feature = "std")]
pub mod jit;
pub mod executable_jit;
pub mod pipeline;
pub mod semantic;
pub mod transform;

pub use clif_module::ClifModule;
pub use compiler::Compiler;
pub use error::{ErrorCode, GlslError, SourceLocation};
pub use executable::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode};
#[cfg(feature = "std")]
pub use jit::JIT;
pub use pipeline::{Backend, CompiledShader, ParseResult, SemanticResult, TransformationPass};
#[cfg(feature = "std")]
pub use pipeline::{CLIFBackend, JITBackend};
pub use transform::FixedPointFormat;
#[cfg(feature = "std")]
pub use transform::FixedPointTransformation;

// Public API functions
pub use compile::glsl_jit;

#[cfg(feature = "emulator")]
pub use compile::glsl_emu_riscv32;
