//! Instruction execution logic.

use crate::{Gpr, Inst};
use super::state::Riscv32Emulator;
use super::types::{StepResult, SyscallInfo};
use super::super::{
    decoder::decode_instruction,
    error::EmulatorError,
    executor::execute_instruction,
};

impl Riscv32Emulator {
    /// Execute a single instruction.
    pub fn step(&mut self) -> Result<StepResult, EmulatorError> {
        // Check instruction limit
        if self.instruction_count >= self.max_instructions {
            return Err(EmulatorError::InstructionLimitExceeded {
                limit: self.max_instructions,
                executed: self.instruction_count,
                pc: self.pc,
                regs: self.regs,
            });
        }

        // Fetch instruction
        let inst_word = self.memory.fetch_instruction(self.pc).map_err(|mut e| {
            match &mut e {
                EmulatorError::InvalidMemoryAccess {
                    regs: err_regs,
                    pc: err_pc,
                    ..
                } => {
                    *err_regs = self.regs;
                    *err_pc = self.pc;
                }
                EmulatorError::UnalignedAccess {
                    regs: err_regs,
                    pc: err_pc,
                    ..
                } => {
                    *err_regs = self.regs;
                    *err_pc = self.pc;
                }
                _ => {}
            }
            e
        })?;

        // Check if compressed instruction (bits [1:0] != 0b11)
        let is_compressed = (inst_word & 0x3) != 0x3;

        // Decode instruction
        let decoded =
            decode_instruction(inst_word).map_err(|reason| EmulatorError::InvalidInstruction {
                pc: self.pc,
                instruction: inst_word,
                reason,
                regs: self.regs,
            })?;

        // Increment instruction count before execution (for cycle counting)
        self.instruction_count += 1;

        // Check if this is a trap BEFORE executing the instruction
        // For EBREAK instructions, we need to check if the current PC is a trap location
        let is_trap_before_execution = if let Inst::Ebreak = decoded {
            // Traps are stored as absolute addresses, compare directly with PC
            self.traps.binary_search_by_key(&self.pc, |(addr, _)| *addr).is_ok()
        } else {
            false
        };

        // Execute instruction
        let exec_result = execute_instruction(decoded, self.pc, &mut self.regs, &mut self.memory)?;

        // Update PC (2 bytes for compressed, 4 for standard)
        let pc_increment = if is_compressed { 2 } else { 4 };
        self.pc = exec_result
            .new_pc
            .unwrap_or(self.pc.wrapping_add(pc_increment));

        // Log instruction with cycle count
        let log_with_cycle = exec_result.log.set_cycle(self.instruction_count);
        self.log_instruction(log_with_cycle);

        // Handle special cases
        if exec_result.should_halt {
            if is_trap_before_execution {
                // This was a trap - find the trap code using the original PC (before PC update)
                let original_pc = self.pc.saturating_sub(pc_increment);
                let index = self.traps.binary_search_by_key(&original_pc, |(addr, _)| *addr)
                    .expect("Trap should be found since is_trap_before_execution was true");
                let trap_code = self.traps[index].1;
                Ok(StepResult::Trap(trap_code))
            } else {
                // Regular ebreak (not a trap)
                Ok(StepResult::Halted)
            }
        } else if exec_result.syscall {
            // Extract syscall info from registers
            let syscall_info = SyscallInfo {
                number: self.regs[Gpr::A7.num() as usize],
                args: [
                    self.regs[Gpr::A0.num() as usize],
                    self.regs[Gpr::A1.num() as usize],
                    self.regs[Gpr::A2.num() as usize],
                    self.regs[Gpr::A3.num() as usize],
                    self.regs[Gpr::A4.num() as usize],
                    self.regs[Gpr::A5.num() as usize],
                    self.regs[Gpr::A6.num() as usize],
                ],
            };
            Ok(StepResult::Syscall(syscall_info))
        } else {
            Ok(StepResult::Continue)
        }
    }
}

