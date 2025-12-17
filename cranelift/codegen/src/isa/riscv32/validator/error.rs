use crate::isa::riscv32::validator::supported::RiscvExtension;
use crate::ir::{Inst, Opcode, Type};
use crate::CodegenError;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

#[derive(Debug)]
pub enum ValidationError {
    UnsupportedInstruction {
        inst: Inst,
        opcode: Opcode,
        reason: String,
    },
    UnsupportedType {
        ty: Type,
        context: String,
    },
    UnsupportedCombination {
        inst: Inst,
        opcode: Opcode,
        types: Vec<Type>,
        reason: String,
    },
    MissingExtension {
        inst: Inst,
        opcode: Opcode,
        required_extension: RiscvExtension,
        reason: String,
    },
}

impl From<ValidationError> for CodegenError {
    fn from(err: ValidationError) -> Self {
        CodegenError::Unsupported(format!("{}", err))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::UnsupportedInstruction { inst, opcode, reason } => {
                write!(f, "Unsupported instruction {} at {:?}: {}", opcode, inst, reason)
            }
            ValidationError::UnsupportedType { ty, context } => {
                write!(f, "Unsupported type {} in {}: not supported on riscv32", ty, context)
            }
            ValidationError::UnsupportedCombination { inst, opcode, types, reason } => {
                write!(f, "Unsupported combination: {} with types {:?} at {:?}: {}",
                      opcode, types, inst, reason)
            }
            ValidationError::MissingExtension { inst, opcode, required_extension, reason } => {
                write!(f, "Missing required extension {} for {} at {:?}: {}",
                      required_extension.name(), opcode, inst, reason)
            }
        }
    }
}
