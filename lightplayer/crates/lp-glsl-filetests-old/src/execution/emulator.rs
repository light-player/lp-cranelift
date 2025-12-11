//! Emulator execution backend

use super::backend::{CompiledCode, ExecutionBackend, ReturnType};
use super::bootstrap::{RESULT_ADDR, STACK_BASE};
use anyhow::Result;
use lp_glsl::FixedPointFormat;
use lp_riscv_tools::Riscv32Emulator;

/// Emulator execution backend
pub struct EmulatorBackend {
    emulator_type: EmulatorType,
}

#[derive(Debug, Clone, Copy)]
pub enum EmulatorType {
    Riscv32,
    // Future: Riscv64, Aarch64, etc.
}

impl EmulatorBackend {
    pub fn new(emulator_type: EmulatorType) -> Self {
        Self { emulator_type }
    }

    fn extract_result_float(
        &self,
        emu: &Riscv32Emulator,
        return_type: ReturnType,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<f32> {
        let memory = emu.memory();

        match return_type {
            ReturnType::Int | ReturnType::Bool => {
                let value = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                match fixed_point_format {
                    Some(FixedPointFormat::Fixed16x16) => Ok(value as f32 / 65536.0),
                    None => Ok(value as f32), // For int tests
                    _ => anyhow::bail!("Unsupported fixed-point format for int return"),
                }
            }
            ReturnType::Float => {
                match fixed_point_format {
                    Some(FixedPointFormat::Fixed16x16) => {
                        // Result is stored as i32 fixed-point, read and convert
                        let value = memory.read_word(RESULT_ADDR).map_err(|e| {
                            anyhow::anyhow!("Failed to read result from memory: {:?}", e)
                        })?;
                        Ok(value as f32 / 65536.0)
                    }
                    Some(FixedPointFormat::Fixed32x32) => {
                        // Result is stored as i64 fixed-point (two words), read and convert
                        let low = memory.read_word(RESULT_ADDR).map_err(|e| {
                            anyhow::anyhow!("Failed to read result from memory: {:?}", e)
                        })?;
                        let high = memory.read_word(RESULT_ADDR + 4).map_err(|e| {
                            anyhow::anyhow!("Failed to read result from memory: {:?}", e)
                        })?;
                        let value = ((high as u32 as u64) << 32) | (low as u32 as u64);
                        let i64_value = value as i64;
                        Ok((i64_value as f64 / 4294967296.0) as f32)
                    }
                    None => {
                        // F32 is stored as i32 bits, read and convert
                        let bits = memory.read_word(RESULT_ADDR).map_err(|e| {
                            anyhow::anyhow!("Failed to read result from memory: {:?}", e)
                        })?;
                        Ok(f32::from_bits(bits as u32))
                    }
                }
            }
            ReturnType::I64 => {
                // Read two words (low and high)
                let low = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let high = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let value = ((high as u32 as u64) << 32) | (low as u32 as u64);
                let i64_value = value as i64;
                match fixed_point_format {
                    Some(FixedPointFormat::Fixed32x32) => {
                        Ok((i64_value as f64 / 4294967296.0) as f32)
                    }
                    None => Ok(i64_value as f32),
                    _ => anyhow::bail!("Unsupported fixed-point format for i64 return"),
                }
            }
            _ => anyhow::bail!(
                "Unsupported return type for extract_result_float: {:?}",
                return_type
            ),
        }
    }

    pub fn run_emulator(&self, binary: &[u8]) -> Result<Riscv32Emulator> {
        match self.emulator_type {
            EmulatorType::Riscv32 => {
                let ram_size = 64 * 1024; // 64KB RAM
                let mut emu = Riscv32Emulator::new(binary.to_vec(), vec![0; ram_size])
                    .with_max_instructions(10_000_000);

                // Set stack pointer
                emu.set_register(lp_riscv_tools::Gpr::Sp, STACK_BASE as i32);

                // Set PC to entry point (0, or _start address)
                emu.set_pc(0);

                // Debug logging (enabled via LP_GLSL_DEBUG env var)
                let debug_enabled = std::env::var("LP_GLSL_DEBUG").is_ok();
                if debug_enabled {
                    eprintln!("[emulator] Starting execution, binary size={} bytes", binary.len());
                }

                // Run until EBREAK
                let mut step_count = 0;
                loop {
                    step_count += 1;
                    match emu.step() {
                        Ok(lp_riscv_tools::StepResult::Halted) => {
                            if debug_enabled {
                                eprintln!("[emulator] Halted after {} steps", step_count);
                            }
                            break;
                        }
                        Ok(lp_riscv_tools::StepResult::Continue) => {
                            if debug_enabled && step_count % 1000 == 0 {
                                let pc = emu.get_pc();
                                eprintln!("[emulator] Step {}, PC=0x{:08x}", step_count, pc);
                            }
                            continue;
                        }
                        Ok(lp_riscv_tools::StepResult::Syscall(_)) => {
                            anyhow::bail!("Unexpected syscall during execution");
                        }
                        Err(e) => {
                            if debug_enabled {
                                let pc = emu.get_pc();
                                eprintln!("[emulator] Error at step {}, PC=0x{:08x}: {:?}", step_count, pc, e);
                            }
                            anyhow::bail!("Emulator error: {:?}", e);
                        }
                    }
                }

                Ok(emu)
            }
        }
    }
}

impl ExecutionBackend for EmulatorBackend {
    fn execute_float(
        &self,
        code: &CompiledCode,
        fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<f32> {
        match code {
            CompiledCode::EmulatorBinary {
                binary,
                return_type,
            } => {
                let emu = self.run_emulator(binary)?;
                self.extract_result_float(&emu, *return_type, fixed_point_format)
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_int(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i32> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                let value = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                Ok(value)
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_i64(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i64> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                let low = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let high = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let value = ((high as u32 as u64) << 32) | (low as u32 as u64);
                Ok(value as i64)
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_bool(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i8> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                let value = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                Ok((value & 0xFF) as i8)
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_vec2(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32)> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                // Read 2 f32 values from RESULT_ADDR
                let bits0 = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let bits1 = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                Ok((f32::from_bits(bits0 as u32), f32::from_bits(bits1 as u32)))
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_vec3(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32)> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                // Read 3 f32 values from RESULT_ADDR
                let bits0 = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let bits1 = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let bits2 = memory
                    .read_word(RESULT_ADDR + 8)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                Ok((
                    f32::from_bits(bits0 as u32),
                    f32::from_bits(bits1 as u32),
                    f32::from_bits(bits2 as u32),
                ))
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_vec4(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                // Read 4 f32 values from RESULT_ADDR
                let bits0 = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let bits1 = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let bits2 = memory
                    .read_word(RESULT_ADDR + 8)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let bits3 = memory
                    .read_word(RESULT_ADDR + 12)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                Ok((
                    f32::from_bits(bits0 as u32),
                    f32::from_bits(bits1 as u32),
                    f32::from_bits(bits2 as u32),
                    f32::from_bits(bits3 as u32),
                ))
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_mat2(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)> {
        // Mat2 is 4 f32 values, same as vec4
        self.execute_vec4(code, _fixed_point_format)
    }

    fn execute_mat3(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                // Read 9 f32 values from RESULT_ADDR
                let mut values = Vec::new();
                for i in 0..9 {
                    let bits = memory
                        .read_word(RESULT_ADDR + (i * 4) as u32)
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to read result from memory: {:?}", e)
                        })?;
                    values.push(f32::from_bits(bits as u32));
                }
                Ok((
                    values[0], values[1], values[2], values[3], values[4], values[5], values[6],
                    values[7], values[8],
                ))
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_mat4(
        &self,
        code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
        f32,
    )> {
        match code {
            CompiledCode::EmulatorBinary { binary, .. } => {
                let emu = self.run_emulator(binary)?;
                let memory = emu.memory();
                // Read 16 f32 values from RESULT_ADDR
                let mut values = Vec::new();
                for i in 0..16 {
                    let bits = memory
                        .read_word(RESULT_ADDR + (i * 4) as u32)
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to read result from memory: {:?}", e)
                        })?;
                    values.push(f32::from_bits(bits as u32));
                }
                Ok((
                    values[0], values[1], values[2], values[3], values[4], values[5], values[6],
                    values[7], values[8], values[9], values[10], values[11], values[12],
                    values[13], values[14], values[15],
                ))
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }
}
