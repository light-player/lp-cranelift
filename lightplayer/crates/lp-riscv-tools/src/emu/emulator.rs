//! Core RISC-V 32-bit emu implementation.

extern crate alloc;

use alloc::{format, string::String, vec::Vec};

use super::{
    decoder::decode_instruction,
    error::EmulatorError,
    executor::execute_instruction,
    logging::{InstLog, LogLevel},
    memory::Memory,
};
use crate::Gpr;
use cranelift_codegen::data_value::DataValue;
use cranelift_codegen::ir::Signature;

/// Default RAM start address (0x80000000, matching embive's RAM_OFFSET).
pub const DEFAULT_RAM_START: u32 = 0x80000000;

/// Result of a single step.
#[derive(Debug, Clone)]
pub enum StepResult {
    /// Normal step completed, continue execution
    Continue,
    /// ECALL encountered, syscall information available
    Syscall(SyscallInfo),
    /// EBREAK encountered, execution halted
    Halted,
}

/// Information about a syscall (ECALL).
#[derive(Debug, Clone)]
pub struct SyscallInfo {
    /// Syscall number (from a7 register)
    pub number: i32,
    /// Syscall arguments (from a0-a6 registers)
    pub args: [i32; 7],
}

/// RISC-V 32-bit emu.
pub struct Riscv32Emulator {
    regs: [i32; 32],
    pc: u32,
    memory: Memory,
    instruction_count: u64,
    max_instructions: u64,
    log_level: LogLevel,
    log_buffer: Vec<InstLog>,
}

impl Riscv32Emulator {
    /// Create a new emu with the given code and RAM.
    ///
    /// # Arguments
    ///
    /// * `code` - Code region (instructions)
    /// * `ram` - RAM region (data)
    pub fn new(code: Vec<u8>, ram: Vec<u8>) -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            memory: Memory::with_default_addresses(code, ram),
            instruction_count: 0,
            max_instructions: 100_000,
            log_level: LogLevel::None,
            log_buffer: Vec::new(),
        }
    }

    /// Set the maximum number of instructions to execute.
    pub fn with_max_instructions(mut self, limit: u64) -> Self {
        self.max_instructions = limit;
        self
    }

    /// Set the logging level.
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

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
            Ok(StepResult::Halted)
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

    /// Run until EBREAK is encountered, returning the value in a0.
    pub fn run_until_ebreak(&mut self) -> Result<i32, EmulatorError> {
        loop {
            match self.step()? {
                StepResult::Halted => {
                    return Ok(self.regs[Gpr::A0.num() as usize]);
                }
                StepResult::Continue => {
                    // Continue execution
                }
                StepResult::Syscall(_) => {
                    // Treat syscall as error in this context (caller should use run_until_ecall)
                    return Err(EmulatorError::InvalidInstruction {
                        pc: self.pc,
                        instruction: 0,
                        reason: String::from("Unexpected ECALL in run_until_ebreak"),
                        regs: self.regs,
                    });
                }
            }
        }
    }

    /// Run until ECALL is encountered, returning syscall information.
    pub fn run_until_ecall(&mut self) -> Result<SyscallInfo, EmulatorError> {
        loop {
            match self.step()? {
                StepResult::Syscall(info) => {
                    return Ok(info);
                }
                StepResult::Halted => {
                    return Err(EmulatorError::InvalidInstruction {
                        pc: self.pc,
                        instruction: 0,
                        reason: String::from("Unexpected EBREAK in run_until_ecall"),
                        regs: self.regs,
                    });
                }
                StepResult::Continue => {
                    // Continue execution
                }
            }
        }
    }

    /// Get the value of a register.
    pub fn get_register(&self, reg: Gpr) -> i32 {
        if reg.num() == 0 {
            0
        } else {
            self.regs[reg.num() as usize]
        }
    }

    /// Set the value of a register.
    ///
    /// Note: Writing to x0 (ZERO) is a no-op.
    pub fn set_register(&mut self, reg: Gpr, value: i32) {
        if reg.num() != 0 {
            self.regs[reg.num() as usize] = value;
        }
    }

    /// Get the current program counter.
    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    /// Set the program counter.
    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    /// Get the number of instructions executed so far.
    pub fn get_instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Get captured log entries.
    pub fn get_logs(&self) -> &[InstLog] {
        &self.log_buffer
    }

    /// Format all captured logs as a string.
    pub fn format_logs(&self) -> String {
        let mut result = String::new();
        for log in &self.log_buffer {
            result.push_str(&format!("{}\n", log));
        }
        result
    }

    /// Clear captured log messages.
    pub fn clear_logs(&mut self) {
        self.log_buffer.clear();
    }

    /// Dump the current emu state as a human-readable string.
    pub fn dump_state(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("PC: 0x{:08x}\n", self.pc));
        result.push_str(&format!(
            "Instructions executed: {}\n",
            self.instruction_count
        ));
        result.push_str("\nRegisters:\n");

        // Named registers first
        let named_regs = [
            (Gpr::Zero, "zero"),
            (Gpr::Ra, "ra"),
            (Gpr::Sp, "sp"),
            (Gpr::Gp, "gp"),
            (Gpr::Tp, "tp"),
            (Gpr::T0, "t0"),
            (Gpr::T1, "t1"),
            (Gpr::T2, "t2"),
            (Gpr::S0, "s0"),
            (Gpr::S1, "s1"),
            (Gpr::A0, "a0"),
            (Gpr::A1, "a1"),
            (Gpr::A2, "a2"),
            (Gpr::A3, "a3"),
            (Gpr::A4, "a4"),
            (Gpr::A5, "a5"),
            (Gpr::A6, "a6"),
            (Gpr::A7, "a7"),
        ];

        for (reg, name) in &named_regs {
            let value = self.get_register(*reg);
            if value != 0 || *reg == Gpr::Zero {
                result.push_str(&format!(
                    "  {} (x{}) = 0x{:08x} ({})\n",
                    name,
                    reg.num(),
                    value as u32,
                    value
                ));
            }
        }

        // Other registers
        for i in 18..32 {
            let reg = Gpr::new(i);
            let value = self.get_register(reg);
            if value != 0 {
                result.push_str(&format!("  x{} = 0x{:08x} ({})\n", i, value as u32, value));
            }
        }

        result
    }

    /// Log an instruction based on the current log level.
    fn log_instruction(&mut self, log: InstLog) {
        match self.log_level {
            LogLevel::None => {}
            LogLevel::Errors => {
                // Only log on errors (handled elsewhere)
            }
            LogLevel::Instructions | LogLevel::Verbose => {
                // Implement rolling buffer: if buffer reaches 100, remove oldest
                if self.log_buffer.len() >= 100 {
                    self.log_buffer.remove(0);
                }
                self.log_buffer.push(log);
            }
        }
    }

    /// Get a reference to the memory (for inspection).
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Get a mutable reference to the memory (for initialization).
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    /// Call a compiled function using the RISC-V calling convention.
    ///
    /// This high-level API handles:
    /// - Setting up arguments in a0-a7 registers (and stack if needed)
    /// - Setting up return address to detect function return
    /// - Executing until function returns
    /// - Extracting return values from a0-a1 (and stack if needed)
    ///
    /// # Arguments
    ///
    /// * `func_entry` - Program counter of function entry point
    /// * `args` - Function arguments as DataValues
    /// * `signature` - Function signature (for ABI conformance)
    ///
    /// # Returns
    ///
    /// Returns the function's return values as DataValues, or an error if execution failed.
    pub fn call_function(
        &mut self,
        func_entry: u32,
        args: &[DataValue],
        signature: &Signature,
    ) -> Result<Vec<DataValue>, EmulatorError> {
        // Reset state for clean function call
        self.regs = [0; 32];
        self.pc = func_entry;
        self.instruction_count = 0;

        // Set up arguments according to RISC-V calling convention
        // a0-a7 (x10-x17) are used for arguments
        // i64 values use register pairs: (low, high)
        let mut arg_reg_idx = 10; // Start at a0 (x10)

        for (i, arg) in args.iter().enumerate() {
            if arg_reg_idx > 17 {
                // Out of argument registers - would need stack, not yet supported
                return Err(EmulatorError::InvalidInstruction {
                    pc: self.pc,
                    instruction: 0,
                    reason: format!("Out of argument registers (arg {})", i),
                    regs: self.regs,
                });
            }

            match arg {
                DataValue::I8(v) => {
                    self.regs[arg_reg_idx] = *v as i32;
                    arg_reg_idx += 1;
                }
                DataValue::I16(v) => {
                    self.regs[arg_reg_idx] = *v as i32;
                    arg_reg_idx += 1;
                }
                DataValue::I32(v) => {
                    self.regs[arg_reg_idx] = *v;
                    arg_reg_idx += 1;
                }
                DataValue::I64(v) => {
                    // i64 uses register pair: (low, high)
                    if arg_reg_idx > 16 {
                        // Need 2 registers, but only 1 available
                        return Err(EmulatorError::InvalidInstruction {
                            pc: self.pc,
                            instruction: 0,
                            reason: format!("Not enough registers for i64 argument (arg {})", i),
                            regs: self.regs,
                        });
                    }
                    let v_u64 = *v as u64;
                    let low = v_u64 as u32 as i32;
                    let high = (v_u64 >> 32) as u32 as i32;
                    self.regs[arg_reg_idx] = low; // Lower register gets low 32 bits
                    self.regs[arg_reg_idx + 1] = high; // Higher register gets high 32 bits
                    arg_reg_idx += 2; // Consume 2 registers
                }
                DataValue::I128(v) => {
                    // i128 uses 4 registers: (reg0, reg1, reg2, reg3)
                    if arg_reg_idx > 14 {
                        // Need 4 registers
                        return Err(EmulatorError::InvalidInstruction {
                            pc: self.pc,
                            instruction: 0,
                            reason: format!("Not enough registers for i128 argument (arg {})", i),
                            regs: self.regs,
                        });
                    }
                    let v_u128 = *v as u128;
                    let reg0 = v_u128 as u32 as i32;
                    let reg1 = (v_u128 >> 32) as u32 as i32;
                    let reg2 = (v_u128 >> 64) as u32 as i32;
                    let reg3 = (v_u128 >> 96) as u32 as i32;
                    self.regs[arg_reg_idx] = reg0;
                    self.regs[arg_reg_idx + 1] = reg1;
                    self.regs[arg_reg_idx + 2] = reg2;
                    self.regs[arg_reg_idx + 3] = reg3;
                    arg_reg_idx += 4; // Consume 4 registers
                }
                _ => {
                    return Err(EmulatorError::InvalidInstruction {
                        pc: self.pc,
                        instruction: 0,
                        reason: format!("Unsupported argument type: {:?}", arg),
                        regs: self.regs,
                    });
                }
            }
        }

        // Set up stack pointer (x2/sp) to end of RAM
        // Leave some space for stack growth
        let ram_size = self.memory.ram().len();
        let stack_top = super::memory::DEFAULT_RAM_START + ram_size as u32;
        self.regs[2] = (stack_top - 16) as i32; // 16-byte aligned, with some space

        // Set up return address (x1/ra) to a special halt address
        // We'll use 0xFFFFFFFC as a sentinel value that triggers halt
        const HALT_ADDRESS: u32 = 0xFFFF_FFFC;
        self.regs[1] = HALT_ADDRESS as i32;

        // Execute until function returns (PC == HALT_ADDRESS or ret instruction)
        loop {
            // Check if we've returned to halt address
            if self.pc == HALT_ADDRESS {
                break;
            }

            match self.step()? {
                StepResult::Halted => {
                    // EBREAK encountered
                    break;
                }
                StepResult::Continue => {
                    // Keep executing
                }
                StepResult::Syscall(_) => {
                    // Unexpected syscall in function execution
                    return Err(EmulatorError::InvalidInstruction {
                        pc: self.pc,
                        instruction: 0,
                        reason: String::from("Unexpected ECALL during function execution"),
                        regs: self.regs,
                    });
                }
            }
        }

        // Extract return values from registers according to signature
        // a0-a7 (x10-x17) are used for return values
        // i64 values use register pairs: (low, high)
        let mut results = Vec::new();
        let mut reg_idx = 10; // Start at a0

        for (i, param) in signature.returns.iter().enumerate() {
            if reg_idx > 17 {
                // Out of return registers - would need stack, not yet supported
                return Err(EmulatorError::InvalidInstruction {
                    pc: self.pc,
                    instruction: 0,
                    reason: format!("Out of return registers (return value {})", i),
                    regs: self.regs,
                });
            }

            // Convert register value to DataValue based on type
            use cranelift_codegen::ir::types;
            let result_value = match param.value_type {
                types::I8 => {
                    let value = DataValue::I8(self.regs[reg_idx] as i8);
                    reg_idx += 1;
                    value
                }
                types::I16 => {
                    let value = DataValue::I16(self.regs[reg_idx] as i16);
                    reg_idx += 1;
                    value
                }
                types::I32 => {
                    let value = DataValue::I32(self.regs[reg_idx]);
                    reg_idx += 1;
                    value
                }
                types::I64 => {
                    if reg_idx > 16 {
                        // Need 2 registers, but only 1 available
                        return Err(EmulatorError::InvalidInstruction {
                            pc: self.pc,
                            instruction: 0,
                            reason: format!(
                                "Not enough registers for i64 return value (return value {})",
                                i
                            ),
                            regs: self.regs,
                        });
                    }
                    // i64 returned in register pair: (low, high)
                    let low = self.regs[reg_idx] as u32 as u64;
                    let high = self.regs[reg_idx + 1] as i64;
                    // Sign-extend the high 32 bits and combine with low 32 bits
                    let value = DataValue::I64((high << 32) | low as i64);
                    reg_idx += 2; // Consumed 2 registers
                    value
                }
                types::I128 => {
                    if reg_idx > 14 {
                        // Need 4 registers
                        return Err(EmulatorError::InvalidInstruction {
                            pc: self.pc,
                            instruction: 0,
                            reason: format!(
                                "Not enough registers for i128 return value (return value {})",
                                i
                            ),
                            regs: self.regs,
                        });
                    }
                    // i128 returned in 4 registers: (reg0, reg1, reg2, reg3)
                    let reg0 = self.regs[reg_idx] as u32 as u128;
                    let reg1 = self.regs[reg_idx + 1] as u32 as u128;
                    let reg2 = self.regs[reg_idx + 2] as u32 as u128;
                    let reg3 = self.regs[reg_idx + 3] as u32 as u128;
                    let value = DataValue::I128(
                        ((reg3 << 96) | (reg2 << 64) | (reg1 << 32) | reg0) as i128,
                    );
                    reg_idx += 4; // Consumed 4 registers
                    value
                }
                _ => {
                    return Err(EmulatorError::InvalidInstruction {
                        pc: self.pc,
                        instruction: 0,
                        reason: format!("Unsupported return type: {:?}", param.value_type),
                        regs: self.regs,
                    });
                }
            };

            results.push(result_value);
        }

        Ok(results)
    }

    /// Helper function to format instructions while skipping long zero runs
    fn format_instructions_with_zero_skip<F>(
        result: &mut String,
        instructions: &[(u32, u32, String)],
        marker_fn: F,
    ) where
        F: Fn(&u32) -> &str,
    {
        use alloc::format;
        const MAX_ZERO_RUN: usize = 16;

        let mut zero_run_start: Option<usize> = None;

        for (idx, (pc, inst_word, disasm)) in instructions.iter().enumerate() {
            if *inst_word == 0 {
                // Track zero runs
                if zero_run_start.is_none() {
                    zero_run_start = Some(idx);
                }
            } else {
                // Non-zero instruction - flush any pending zero run
                if let Some(zero_start) = zero_run_start.take() {
                    let zero_count = idx - zero_start;
                    if zero_count > MAX_ZERO_RUN {
                        // Summarize long zero runs
                        result.push_str(&format!(
                            "  ... ({} zero instructions skipped)\n",
                            zero_count
                        ));
                    } else {
                        // Show short zero runs
                        for i in 0..zero_count {
                            let zero_pc = instructions[zero_start + i].0;
                            let marker = marker_fn(&zero_pc);
                            result.push_str(&format!(
                                "{}{:3}: 0x{:08x}: {}\n",
                                marker,
                                zero_start + i,
                                zero_pc,
                                instructions[zero_start + i].2
                            ));
                        }
                    }
                }

                // Format non-zero instruction
                let marker = marker_fn(pc);
                result.push_str(&format!("{}{:3}: 0x{:08x}: {}\n", marker, idx, pc, disasm));
            }
        }

        // Flush any remaining zero run at the end
        if let Some(zero_start) = zero_run_start {
            let zero_count = instructions.len() - zero_start;
            if zero_count > MAX_ZERO_RUN {
                result.push_str(&format!(
                    "  ... ({} zero instructions skipped)\n",
                    zero_count
                ));
            } else {
                for i in 0..zero_count {
                    let zero_pc = instructions[zero_start + i].0;
                    let marker = marker_fn(&zero_pc);
                    result.push_str(&format!(
                        "{}{:3}: 0x{:08x}: {}\n",
                        marker,
                        zero_start + i,
                        zero_pc,
                        instructions[zero_start + i].2
                    ));
                }
            }
        }
    }

    /// Format debug information including disassembly and execution logs.
    ///
    /// # Arguments
    ///
    /// * `highlight_pc` - Optional PC to highlight in disassembly (for errors)
    /// * `log_count` - Number of recent logs to show (default 20)
    pub fn format_debug_info(&self, highlight_pc: Option<u32>, log_count: usize) -> String {
        use alloc::format;

        let mut result = String::new();
        let code = self.memory.code();

        // Disassemble all instructions
        let mut instructions = Vec::new();
        for i in (0..code.len()).step_by(4) {
            if i + 4 <= code.len() {
                let inst_word =
                    u32::from_le_bytes([code[i], code[i + 1], code[i + 2], code[i + 3]]);
                let pc = i as u32;
                // Use proper disassembly formatting
                let disasm = crate::inst::format_instruction(inst_word);
                instructions.push((pc, inst_word, disasm));
            }
        }

        // Show disassembly
        result.push_str("Disassembly:\n");
        if let Some(error_pc) = highlight_pc {
            if instructions.len() > 50 {
                // Find the index of the highlighted instruction
                let fail_idx = instructions
                    .iter()
                    .position(|(pc, _, _)| *pc == error_pc)
                    .unwrap_or(0);
                let start = fail_idx.saturating_sub(10);
                let end = (fail_idx + 11).min(instructions.len());

                if start > 0 {
                    result.push_str("  ...\n");
                }

                for (idx, (pc, _inst_word, disasm)) in instructions[start..end].iter().enumerate() {
                    let actual_idx = start + idx;
                    let marker = if *pc == error_pc { ">>> " } else { "    " };
                    result.push_str(&format!(
                        "{}{:3}: 0x{:08x}: {}\n",
                        marker, actual_idx, pc, disasm
                    ));
                }

                if end < instructions.len() {
                    result.push_str("  ...\n");
                }
            } else {
                // Show all instructions, skipping long zero runs
                let error_pc = error_pc; // Capture for closure
                Self::format_instructions_with_zero_skip(&mut result, &instructions, move |pc| {
                    if *pc == error_pc { ">>> " } else { "    " }
                });
            }
        } else {
            // No highlight - show recent instructions or all if small
            if instructions.len() > 50 {
                let start = instructions.len().saturating_sub(20);
                if start > 0 {
                    result.push_str("  ...\n");
                }
                for (idx, (pc, _inst_word, disasm)) in instructions[start..].iter().enumerate() {
                    let actual_idx = start + idx;
                    result.push_str(&format!("   {:3}: 0x{:08x}: {}\n", actual_idx, pc, disasm));
                }
            } else {
                // Show all instructions, skipping long zero runs
                Self::format_instructions_with_zero_skip(&mut result, &instructions, |_| "   ");
            }
        }

        // Show logs
        if !self.log_buffer.is_empty() {
            result.push_str("\nLast execution logs:\n");
            let start = self.log_buffer.len().saturating_sub(log_count);
            for log in &self.log_buffer[start..] {
                result.push_str(&format!("{}\n", log));
            }
        }

        result
    }
}
