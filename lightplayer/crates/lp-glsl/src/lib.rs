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
mod compiler;
mod ir;
mod ir_utils;
#[cfg(feature = "std")]
mod test_utils;
mod util;

// Re-exports
#[cfg(feature = "emulator")]
pub use backend::GlslEmulatorModule;
pub use backend::GlslJitModule;
pub use backend::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode};
pub use compiler::{
    parse_program_with_registry, Backend, CompilationPipeline, CompiledShader, GlslCompiler, ParseResult,
    SemanticResult, TransformationPass,
};

/// Type alias for convenience
pub type Compiler = GlslCompiler;
pub use error::{ErrorCode, GlslError, SourceLocation};
pub use ir::ClifModule;
pub use semantic::type_check::inference::infer_expr_type_in_context;
pub use transform::FixedPointFormat;
pub use transform::fixed32::transform_module;

// Public API functions
pub use compiler::glsl_jit;

#[cfg(feature = "emulator")]
pub use compiler::{glsl_emu_riscv32, glsl_emu_riscv32_with_metadata};

#[cfg(feature = "std")]
pub use test_utils::execute_main;
