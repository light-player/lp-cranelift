//! RISC-V 32-bit validation module
//!
//! This module provides validation for CLIF IR before it is lowered to machine code.
//! It ensures that only supported instructions and types are used, and that required
//! CPU extensions are enabled for the instructions that need them.

use crate::ir::Function;
use crate::isa::riscv32::Riscv32Backend;
use crate::CodegenResult;

pub mod error;
pub mod instruction;
pub mod supported;
pub mod types;

/// Main validator struct
pub struct Validator<'a> {
    backend: &'a Riscv32Backend,
}

impl<'a> Validator<'a> {
    /// Create a new validator for the given backend
    pub fn new(backend: &'a Riscv32Backend) -> Self {
        Self { backend }
    }

    /// Validate an entire function
    pub fn validate_function(&self, func: &Function) -> CodegenResult<()> {
        // Validate types first (this catches unsupported types early)
        self.validate_types(func)?;

        // Then validate instructions
        self.validate_instructions(func)?;

        Ok(())
    }

    /// Validate all types in a function
    fn validate_types(&self, func: &Function) -> CodegenResult<()> {
        types::validate_types(func)
    }

    /// Validate all instructions in a function
    fn validate_instructions(&self, func: &Function) -> CodegenResult<()> {
        instruction::validate_instructions(func, &self.backend)
    }

    /// Check if a specific RISC-V extension is enabled
    fn check_extension(&self, ext: supported::RiscvExtension) -> bool {
        match ext {
            supported::RiscvExtension::I => true, // Always required
            supported::RiscvExtension::M => self.backend.isa_flags.has_m(),
            supported::RiscvExtension::F => self.backend.isa_flags.has_f(),
            supported::RiscvExtension::D => self.backend.isa_flags.has_d(),
            supported::RiscvExtension::A => self.backend.isa_flags.has_a(),
            // C extension is split into sub-extensions; zca is the base compressed extension
            supported::RiscvExtension::C => self.backend.isa_flags.has_zca(),
            supported::RiscvExtension::Zba => self.backend.isa_flags.has_zba(),
            supported::RiscvExtension::Zbb => self.backend.isa_flags.has_zbb(),
            supported::RiscvExtension::Zbc => self.backend.isa_flags.has_zbc(),
            supported::RiscvExtension::Zbs => self.backend.isa_flags.has_zbs(),
            supported::RiscvExtension::Zca => self.backend.isa_flags.has_zca(),
            supported::RiscvExtension::Zcb => self.backend.isa_flags.has_zcb(),
            supported::RiscvExtension::Zcd => self.backend.isa_flags.has_zcd(),
            supported::RiscvExtension::Zcf => self.backend.isa_flags.has_zcf(),
            supported::RiscvExtension::Zfa => self.backend.isa_flags.has_zfa(),
            supported::RiscvExtension::Zfh => self.backend.isa_flags.has_zfh(),
            supported::RiscvExtension::Zfhmin => self.backend.isa_flags.has_zfhmin(),
            supported::RiscvExtension::Zicsr => self.backend.isa_flags.has_zicsr(),
            supported::RiscvExtension::Zifencei => self.backend.isa_flags.has_zifencei(),
            supported::RiscvExtension::Zicond => self.backend.isa_flags.has_zicond(),
            supported::RiscvExtension::Zbkb => self.backend.isa_flags.has_zbkb(),
            supported::RiscvExtension::Zbkc => self.backend.isa_flags.has_zbkc(),
            supported::RiscvExtension::Zbkx => self.backend.isa_flags.has_zbkx(),
            supported::RiscvExtension::Zkn => self.backend.isa_flags.has_zkn(),
            supported::RiscvExtension::Zks => self.backend.isa_flags.has_zks(),
            supported::RiscvExtension::V => self.backend.isa_flags.has_v(),
            supported::RiscvExtension::Zvfh => self.backend.isa_flags.has_zvfh(),
        }
    }
}

/// Public validation function that creates a validator and validates a function
pub fn validate_function(backend: &Riscv32Backend, func: &Function) -> CodegenResult<()> {
    let validator = Validator::new(backend);
    validator.validate_function(func)
}
