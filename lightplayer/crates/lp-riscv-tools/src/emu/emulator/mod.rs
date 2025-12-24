//! RISC-V 32-bit emulator implementation.
//!
//! This module contains the emulator implementation broken down into logical submodules:
//! - `types`: Public types (StepResult, SyscallInfo)
//! - `state`: Core state and initialization
//! - `registers`: Register and PC management
//! - `execution`: Instruction execution
//! - `function_call`: Function calling with ABI setup
//! - `run_loops`: High-level run methods
//! - `debug`: Debug formatting and logging

mod types;
mod state;
mod registers;
mod execution;
mod function_call;
mod run_loops;
mod debug;

pub use types::{StepResult, SyscallInfo};
pub use state::{Riscv32Emulator, DEFAULT_RAM_START};

