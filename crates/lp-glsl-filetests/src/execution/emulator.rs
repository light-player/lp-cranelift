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

    fn extract_result(
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
                // F32 is stored as i32 bits, read and convert
                let bits = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                Ok(f32::from_bits(bits as u32))
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
            _ => anyhow::bail!("Unsupported return type for emulator: {:?}", return_type),
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
                match self.emulator_type {
                    EmulatorType::Riscv32 => {
                        // Create emulator with binary as code
                        let ram_size = 64 * 1024; // 64KB RAM
                        let mut emu = Riscv32Emulator::new(binary.clone(), vec![0; ram_size])
                            .with_max_instructions(10_000_000);

                        // Set stack pointer
                        emu.set_register(lp_riscv_tools::Gpr::Sp, STACK_BASE as i32);

                        // Set PC to entry point (0, or _start address)
                        emu.set_pc(0);

                        // Run until EBREAK
                        loop {
                            match emu.step() {
                                Ok(lp_riscv_tools::StepResult::Halted) => break,
                                Ok(lp_riscv_tools::StepResult::Continue) => continue,
                                Ok(lp_riscv_tools::StepResult::Syscall(_)) => {
                                    anyhow::bail!("Unexpected syscall during execution");
                                }
                                Err(e) => anyhow::bail!("Emulator error: {:?}", e),
                            }
                        }

                        // Extract result from memory
                        // TODO: Add read_memory method to Riscv32Emulator
                        // For now, use workaround
                        self.extract_result(&emu, *return_type, fixed_point_format)
                    }
                }
            }
            _ => anyhow::bail!("EmulatorBackend requires EmulatorBinary compiled code"),
        }
    }

    fn execute_int(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i32> {
        // Similar to execute_float but extract i32
        anyhow::bail!("execute_int not yet implemented for emulator");
    }

    fn execute_i64(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i64> {
        anyhow::bail!("execute_i64 not yet implemented for emulator");
    }

    fn execute_bool(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<i8> {
        anyhow::bail!("execute_bool not yet implemented for emulator");
    }

    fn execute_vec2(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32)> {
        anyhow::bail!("execute_vec2 not yet implemented for emulator");
    }

    fn execute_vec3(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32)> {
        anyhow::bail!("execute_vec3 not yet implemented for emulator");
    }

    fn execute_vec4(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)> {
        anyhow::bail!("execute_vec4 not yet implemented for emulator");
    }

    fn execute_mat2(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32)> {
        anyhow::bail!("execute_mat2 not yet implemented for emulator");
    }

    fn execute_mat3(
        &self,
        _code: &CompiledCode,
        _fixed_point_format: Option<FixedPointFormat>,
    ) -> Result<(f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
        anyhow::bail!("execute_mat3 not yet implemented for emulator");
    }

    fn execute_mat4(
        &self,
        _code: &CompiledCode,
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
        anyhow::bail!("execute_mat4 not yet implemented for emulator");
    }
}
