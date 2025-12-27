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

mod debug;
mod execution;
mod function_call;
mod registers;
mod run_loops;
mod state;
mod types;

pub use state::{Riscv32Emulator, DEFAULT_RAM_START};
pub use types::{StepResult, SyscallInfo};
