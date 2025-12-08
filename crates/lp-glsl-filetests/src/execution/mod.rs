//! Execution backends for running compiled GLSL code
//!
//! Provides abstraction for different execution methods:
//! - Native JIT execution (host machine)
//! - Emulator execution (riscv32, future: riscv64, etc.)

pub mod backend;
pub mod binary;
pub mod bootstrap;
pub mod emulator;
pub mod native;

pub use backend::{CompiledCode, ExecutionBackend, ReturnType};
pub use emulator::{EmulatorBackend, EmulatorType};
pub use native::NativeJitBackend;
