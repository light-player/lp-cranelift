//! RISC-V 32-bit emulator and instruction utilities.
//!
//! This crate provides:
//! - RISC-V 32-bit emulator for testing generated code
//! - Instruction encoding/decoding utilities
//! - Register and instruction definitions
//!
//! Note: Assembly parsing and disassembly have been removed.
//! Use Capstone for disassembly instead.

#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// Debug macro
#[macro_use]
mod debug;

// Emulator modules
pub mod emu;

// ELF loading utility
#[cfg(feature = "std")]
pub mod elf_loader;

// ELF linking utility
#[cfg(feature = "std")]
pub mod elf_linker;

// Lazy linking utility
#[cfg(feature = "std")]
pub mod lazy_linking;

// Executable linker utility
#[cfg(feature = "std")]
pub mod executable_linker;

// Instruction utilities
pub mod auipc_imm;
pub mod decode;
pub mod decode_rvc;
pub mod encode;
pub mod format;
pub mod inst;
// pub mod inst_buffer;  // TODO: Adapt to use cranelift types or remove
pub mod register_role;
pub mod regs;

// Re-exports for convenience
pub use decode::decode_instruction;
pub use emu::{
    EmulatorError, InstLog, LogLevel, MemoryAccessKind, Riscv32Emulator, StepResult, SyscallInfo,
};
pub use inst::{Inst, format_instruction};
pub use regs::Gpr;

#[cfg(feature = "std")]
pub use elf_loader::{ElfLoadInfo, find_symbol_address, load_elf};
#[cfg(feature = "std")]
pub use elf_linker::{link_static_library, LinkerError};
#[cfg(feature = "std")]
pub use executable_linker::link_into_executable;
