//! Executable GLSL module trait and implementations
//!
//! This module provides a trait-based API for executing GLSL functions that
//! abstracts away JIT vs Emulator implementations.

use crate::backend::glsl_value::GlslValue;
use crate::error::GlslError;
use crate::semantic::functions::FunctionSignature;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

/// Trait for executing GLSL functions with various return types
/// Abstracts away JIT vs Emulator implementations
///
/// **Current State**: Supports basic function calling with in-parameters only.
/// Future extensions will add:
/// - Uniform variables (`set_uniform`, `get_uniform`, `list_uniforms`)
/// - Texture/sampler binding (`bind_texture`, `bind_sampler`)
/// - Built-in variables (`set_builtin`, e.g., `gl_Position`, `gl_FragCoord`)
/// - `out` and `inout` parameters
pub trait GlslExecutable {
    /// Call a function that returns void
    fn call_void(&mut self, name: &str, args: &[GlslValue]) -> Result<(), GlslError>;

    /// Call a function that returns i32
    fn call_i32(&mut self, name: &str, args: &[GlslValue]) -> Result<i32, GlslError>;

    /// Call a function that returns f32
    fn call_f32(&mut self, name: &str, args: &[GlslValue]) -> Result<f32, GlslError>;

    /// Call a function that returns bool
    fn call_bool(&mut self, name: &str, args: &[GlslValue]) -> Result<bool, GlslError>;

    /// Call a function that returns a vector (vec2, vec3, or vec4)
    /// `dim` is the dimension (2, 3, or 4)
    fn call_vec(
        &mut self,
        name: &str,
        args: &[GlslValue],
        dim: usize,
    ) -> Result<Vec<f32>, GlslError>;

    /// Call a function that returns a matrix
    /// `rows` and `cols` specify the matrix dimensions (e.g., 2x2, 3x3, 4x4)
    /// Returns a flat vector in column-major order
    fn call_mat(
        &mut self,
        name: &str,
        args: &[GlslValue],
        rows: usize,
        cols: usize,
    ) -> Result<Vec<f32>, GlslError>;

    /// Get the signature of a function by name
    fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature>;

    /// List all available function names
    fn list_functions(&self) -> Vec<String>;

    /// Get emulator state as a formatted string, if this is an emulator module.
    /// Returns None for non-emulator implementations (e.g., JIT).
    #[cfg(feature = "std")]
    fn format_emulator_state(&self) -> Option<String> {
        None
    }

    // TODO: Future extensions:
    // fn set_uniform(&mut self, name: &str, value: GlslValue) -> Result<(), GlslError>;
    // fn get_uniform(&self, name: &str) -> Option<&GlslValue>;
    // fn list_uniforms(&self) -> Vec<String>;
    // fn bind_texture(&mut self, unit: u32, texture: Texture) -> Result<(), GlslError>;
    // fn bind_sampler(&mut self, unit: u32, sampler: Sampler) -> Result<(), GlslError>;
    // fn set_builtin(&mut self, name: &str, value: GlslValue) -> Result<(), GlslError>;
}

/// Execution mode for GLSL compilation
#[derive(Debug, Clone)]
pub enum RunMode {
    /// JIT compile and execute on the host architecture
    HostJit,
    /// Emulate execution (currently RISC-V 32-bit only)
    /// Requires `emulator` feature flag to be enabled
    Emulator {
        /// Maximum memory size in bytes (RAM)
        max_memory: usize,
        /// Stack size in bytes
        stack_size: usize,
        /// Maximum instruction count before timeout
        max_instructions: u64,
    },
}

/// Decimal format for floating-point operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalFormat {
    /// Native floating-point (f32/f64)
    Float,
    /// Fixed-point 32-bit (Q format)
    Fixed32,
    /// Fixed-point 64-bit (not yet supported)
    Fixed64,
}

/// Compilation options
#[derive(Debug, Clone)]
pub struct GlslOptions {
    pub run_mode: RunMode,
    pub decimal_format: DecimalFormat,
}

impl GlslOptions {
    pub fn validate(&self) -> Result<(), GlslError> {
        use crate::error::{ErrorCode, GlslError};
        use target_lexicon::Triple;

        // Validate option combinations
        match (&self.run_mode, self.decimal_format) {
            (RunMode::Emulator { .. }, DecimalFormat::Float) => {
                // TODO: Float support will be added for riscv32_imafc in the future
                Err(GlslError::new(
                    ErrorCode::E0400,
                    "Float format not yet supported in emulator mode (will be supported for riscv32_imafc)",
                ))
            }
            (RunMode::HostJit, DecimalFormat::Float) => {
                // Check if host supports float by checking triple string
                let triple = Triple::host();
                let arch_str = format!("{:?}", triple.architecture);
                if arch_str.contains("riscv32") {
                    Err(GlslError::new(
                        ErrorCode::E0400,
                        "Float format not supported on RISC-V 32-bit",
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Default options for JIT execution
    pub fn jit() -> Self {
        Self {
            run_mode: RunMode::HostJit,
            decimal_format: DecimalFormat::Float,
        }
    }

    /// Default options for emulator execution
    pub fn emulator(max_memory: usize, stack_size: usize) -> Self {
        Self {
            run_mode: RunMode::Emulator {
                max_memory,
                stack_size,
                max_instructions: 10_000_000,
            },
            decimal_format: DecimalFormat::Fixed32,
        }
    }

    /// Convenience constructor for RISC-V 32-bit IMA(C) emulator
    /// Uses 1MB RAM, 64KB stack, and Fixed32 format
    #[cfg(feature = "emulator")]
    pub fn emu_riscv32_imac() -> Self {
        Self {
            run_mode: RunMode::Emulator {
                max_memory: 1024 * 1024, // 1MB
                stack_size: 64 * 1024,   // 64KB
                max_instructions: 10_000_000,
            },
            decimal_format: DecimalFormat::Fixed32,
        }
    }
}

// ============================================================================
// Module implementations (in separate files)
// ============================================================================

// Re-export module types for convenience
// Note: GlslEmulatorModule is conditionally compiled and may not be used in all builds
#[cfg(feature = "emulator")]
#[allow(unused_imports)]
pub use crate::backend::emu::GlslEmulatorModule;
