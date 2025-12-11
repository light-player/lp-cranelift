//! GLSL compilation logic.
//!
//! This module contains the core compilation components that transform GLSL source
//! into Cranelift IR, including parsing, semantic analysis, code generation, and linking.

pub(crate) mod glsl_compiler;
pub(crate) mod link;
pub(crate) mod pipeline;

// Re-exports used by crate root; suppress unused warnings within this module.
#[allow(unused_imports)]
pub use glsl_compiler::GlslCompiler;
#[allow(unused_imports)]
pub use link::rebuild_function_for_module;
#[allow(unused_imports)]
pub use pipeline::{
    Backend, CompilationPipeline, CompiledShader, ParseResult, SemanticResult, TransformationPass,
};

// Re-export create_minimal_module_for_declarations for internal use
#[allow(unused_imports)]
pub(crate) use glsl_compiler::create_minimal_module_for_declarations;

// ============================================================================
// Public API functions
// ============================================================================

use crate::backend::executable::{GlslExecutable, GlslOptions, RunMode};
use crate::error::GlslError;
use crate::ir::ClifModule;
use crate::transform::fixed32::{FixedPointFormat, transform_module};
use cranelift_codegen::isa::OwnedTargetIsa;

#[cfg(feature = "std")]
use cranelift_native;

#[cfg(not(feature = "std"))]
use alloc::format as alloc_format;
#[cfg(feature = "std")]
use std::format as alloc_format;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};
#[cfg(feature = "std")]
use std::{boxed::Box, string::String};

/// Compile GLSL to CLIF module (internal, reusable)
/// This is the core compilation step that can be reused for different backends
pub fn compile_glsl_to_clif(source: &str, options: &GlslOptions) -> Result<ClifModule, GlslError> {
    use crate::backend::executable::DecimalFormat;

    options.validate()?;

    let mut compiler = GlslCompiler::new();

    // Determine ISA based on run mode
    let isa = match &options.run_mode {
        #[cfg(feature = "std")]
        RunMode::HostJit => create_host_isa()?,
        #[cfg(all(feature = "emulator", feature = "std"))]
        RunMode::Emulator { .. } => create_riscv32_isa()?,
        #[cfg(not(feature = "emulator"))]
        RunMode::Emulator { .. } => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "Emulator mode requires 'emulator' feature flag",
            ));
        }
        #[cfg(all(not(feature = "std"), feature = "emulator"))]
        RunMode::HostJit => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "HostJit mode requires 'std' feature flag",
            ));
        }
    };

    // Compile to CLIF
    let mut module = compiler.compile_to_clif_module(source, isa)?;

    // Apply transformations
    match options.decimal_format {
        DecimalFormat::Fixed32 => {
            module = transform_module(&module, FixedPointFormat::Fixed16x16)?;
        }
        DecimalFormat::Fixed64 => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "Fixed64 not yet supported",
            ));
        }
        DecimalFormat::Float => {
            // No transformation needed
        }
    }

    Ok(module)
}

/// Compile and JIT execute GLSL
/// Works in both std and no_std environments
pub fn glsl_jit(source: &str, options: GlslOptions) -> Result<Box<dyn GlslExecutable>, GlslError> {
    let module = compile_glsl_to_clif(source, &options)?;
    let jit_module = link::link_glsl_for_jit(module)?;
    Ok(Box::new(jit_module))
}

/// Compile and execute GLSL in RISC-V 32-bit emulator
/// Requires `emulator` feature flag to be enabled
#[cfg(feature = "emulator")]
pub fn glsl_emu_riscv32(
    source: &str,
    options: GlslOptions,
) -> Result<Box<dyn GlslExecutable>, GlslError> {
    use link::EmulatorOptions;

    let module = compile_glsl_to_clif(source, &options)?;

    let emulator_options = match &options.run_mode {
        RunMode::Emulator {
            max_memory,
            stack_size,
            max_instructions,
            ..
        } => EmulatorOptions {
            max_memory: *max_memory,
            stack_size: *stack_size,
            max_instructions: *max_instructions,
        },
        _ => {
            return Err(GlslError::new(
                crate::error::ErrorCode::E0400,
                "Invalid run mode for emulator",
            ));
        }
    };

    let emu_module = link::link_glsl_for_emulator(module, &emulator_options)?;
    Ok(Box::new(emu_module))
}

/// Create host ISA for JIT compilation
#[cfg(feature = "std")]
fn create_host_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_codegen::settings::{self, Configurable};

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to set is_pic: {}", e),
        )
    })?;
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set use_colocated_libcalls: {}", e),
            )
        })?;
    flag_builder
        .set("enable_multi_ret_implicit_sret", "true")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set enable_multi_ret_implicit_sret: {}", e),
            )
        })?;

    let flags = settings::Flags::new(flag_builder);
    let isa_builder = cranelift_native::builder().map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            format!("host machine is not supported: {}", e),
        )
    })?;
    isa_builder.finish(flags).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to create host ISA: {}", e),
        )
    })
}

/// Create RISC-V 32-bit ISA for emulator compilation
#[cfg(feature = "emulator")]
fn create_riscv32_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_codegen::isa::riscv32::isa_builder;
    use cranelift_codegen::settings::{self, Configurable};
    use target_lexicon::{
        Architecture, BinaryFormat, Environment, OperatingSystem, Riscv32Architecture, Triple,
        Vendor,
    };

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to set is_pic: {}", e),
        )
    })?;
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set use_colocated_libcalls: {}", e),
            )
        })?;
    flag_builder
        .set("enable_multi_ret_implicit_sret", "true")
        .map_err(|e| {
            GlslError::new(
                crate::error::ErrorCode::E0400,
                alloc_format!("failed to set enable_multi_ret_implicit_sret: {}", e),
            )
        })?;

    let flags = settings::Flags::new(flag_builder);
    let triple = Triple {
        architecture: Architecture::Riscv32(Riscv32Architecture::Riscv32imac),
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::None_,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf,
    };

    isa_builder(triple).finish(flags).map_err(|e| {
        GlslError::new(
            crate::error::ErrorCode::E0400,
            alloc_format!("failed to create riscv32 ISA: {}", e),
        )
    })
}
