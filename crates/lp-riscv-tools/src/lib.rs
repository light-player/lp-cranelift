//! RISC-V 32-bit emulator and instruction utilities.
//!
//! This crate provides:
//! - RISC-V 32-bit emulator for testing generated code
//! - Instruction encoding/decoding utilities
//! - Assembly parsing and disassembly
//! - Register and instruction definitions

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Emulator modules
pub mod emu;

// Instruction utilities
pub mod asm_parser;
pub mod auipc_imm;
pub mod decode;
pub mod disasm;
pub mod encode;
pub mod format;
pub mod inst;
pub mod inst_buffer;
pub mod register_role;
pub mod regs;

// Re-exports
pub use emu::{
    Riscv32Emulator, StepResult, SyscallInfo,
    EmulatorError, MemoryAccessKind,
    InstLog, LogLevel,
};
pub use regs::Gpr;

