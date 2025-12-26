//! GLSL fragment shader compiler using Cranelift JIT.
//!
//! Phase 1: Basic architecture with int/bool support only.

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

// Always declare alloc so we can use alloc::string::String etc. in both std and no_std modes
#[macro_use]
extern crate alloc;

pub mod error;
pub mod frontend;

// Private modules
mod exec;
mod backend;
mod backend2;

// Re-exports
#[cfg(feature = "emulator")]
pub use exec::GlslEmulatorModule;
pub use exec::GlslJitModule;
pub use exec::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode};
pub use frontend::{
    parse_program_with_registry, Backend, CompilationPipeline, CompiledShader, GlslCompiler, ParseResult,
    SemanticResult, TransformationPass,
};
pub use frontend::codegen;
pub use frontend::semantic;
pub use frontend::intrinsics;

/// Type alias for convenience
pub type Compiler = GlslCompiler;
pub use error::{ErrorCode, GlslError};
pub use backend::ir::ClifModule;
pub use frontend::semantic::type_check::inference::infer_expr_type_in_context;
pub use backend::transform::FixedPointFormat;
pub use backend::transform::fixed32::transform_module;

// Public API functions
pub use frontend::glsl_jit;

#[cfg(feature = "emulator")]
pub use frontend::{glsl_emu_riscv32, glsl_emu_riscv32_with_metadata};

#[cfg(feature = "std")]
pub use exec::execute_main::execute_main;
pub use frontend::src_loc::GlSourceLoc;
