//! Execution backends for GLSL shaders.
//!
//! This module provides trait-based APIs and implementations for executing
//! compiled GLSL shaders, abstracting away JIT vs Emulator implementations.

pub(crate) mod executable;
pub(crate) mod jit;
#[cfg(feature = "emulator")]
pub(crate) mod emu;

pub use executable::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode};
pub use jit::GlslJitModule;
#[cfg(feature = "emulator")]
pub use emu::GlslEmulatorModule;

