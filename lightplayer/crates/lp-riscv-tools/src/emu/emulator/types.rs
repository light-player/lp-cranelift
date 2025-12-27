//! Public types for the RISC-V 32-bit emulator.

use cranelift_codegen::ir::TrapCode;

/// Result of a single step.
#[derive(Debug, Clone)]
pub enum StepResult {
    /// Normal step completed, continue execution
    Continue,
    /// ECALL encountered, syscall information available
    Syscall(SyscallInfo),
    /// EBREAK encountered, execution halted
    Halted,
    /// Trap encountered with trap code
    Trap(TrapCode),
}

/// Information about a syscall (ECALL).
#[derive(Debug, Clone)]
pub struct SyscallInfo {
    /// Syscall number (from a7 register)
    pub number: i32,
    /// Syscall arguments (from a0-a6 registers)
    pub args: [i32; 7],
}
