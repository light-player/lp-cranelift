use super::error::ValidationError;
use super::supported;
use crate::CodegenResult;
use crate::ir::{DataFlowGraph, Function, Inst, Opcode, types::*};
use alloc::format;
use alloc::string::ToString;
use alloc::{vec, vec::Vec};

/// Validate all instructions in a function
pub fn validate_instructions(
    func: &Function,
    backend: &super::super::Riscv32Backend,
) -> CodegenResult<()> {
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            validate_instruction(func, inst, backend)?;
        }
    }
    Ok(())
}

/// Validate a single instruction
pub fn validate_instruction(
    func: &Function,
    inst: Inst,
    backend: &super::super::Riscv32Backend,
) -> CodegenResult<()> {
    let opcode = func.dfg.insts[inst].opcode();
    let data = &func.dfg.insts[inst];

    // Collect all types involved in this instruction
    let mut types = Vec::new();

    // Add result types
    for &result in func.dfg.inst_results(inst) {
        types.push(func.dfg.value_type(result));
    }

    // Add argument types
    for &arg in func.dfg.inst_args(inst) {
        types.push(func.dfg.value_type(arg));
    }

    // Get all required extensions for this opcode + types combination
    // This handles both opcode-level and type-level requirements in one place
    let required_exts = supported::required_extensions(opcode, &types);

    // Note: required_extensions() already combines opcode and type requirements,
    // so we don't need to call type_required_extensions() separately here

    // Check if all required extensions are enabled
    // STRICT: Reject if any required extension is not enabled
    for ext in required_exts {
        if !check_extension(backend, ext) {
            return Err(ValidationError::MissingExtension {
                inst,
                opcode,
                required_extension: ext,
                reason: format!(
                    "{} requires {} extension ({}), but it is not enabled. \
                     Enable {} extension in target flags to use this instruction.",
                    opcode,
                    ext.name(),
                    ext.description(),
                    ext.name()
                ),
            }
            .into());
        }
    }

    // Additional opcode-specific validation (beyond extension checks)
    // This is for things like "i64 division not yet implemented" etc.
    match opcode {
        Opcode::Iadd => validate_iadd(func, inst, &func.dfg)?,
        Opcode::Sdiv => validate_sdiv(func, inst, &func.dfg)?,
        Opcode::Fadd => validate_fadd(func, inst, &func.dfg)?,
        Opcode::Udiv => validate_udiv(func, inst, &func.dfg)?,
        Opcode::Urem => validate_urem(func, inst, &func.dfg)?,
        Opcode::Srem => validate_srem(func, inst, &func.dfg)?,
        Opcode::Bswap => validate_bswap(func, inst, &func.dfg)?,
        Opcode::Bitrev => validate_bitrev(func, inst, &func.dfg)?,
        Opcode::UaddOverflow => validate_overflow_instruction(func, inst, opcode, &func.dfg)?,
        Opcode::SaddOverflow => validate_overflow_instruction(func, inst, opcode, &func.dfg)?,
        Opcode::UsubOverflow => validate_overflow_instruction(func, inst, opcode, &func.dfg)?,
        Opcode::SsubOverflow => validate_overflow_instruction(func, inst, opcode, &func.dfg)?,
        Opcode::UmulOverflow => validate_overflow_instruction(func, inst, opcode, &func.dfg)?,
        Opcode::SmulOverflow => validate_overflow_instruction(func, inst, opcode, &func.dfg)?,
        Opcode::Bmask => validate_bmask(func, inst, &func.dfg)?,

        // ... other opcodes
        _ => {
            // Check if opcode is in supported list
            if !supported::is_opcode_supported(opcode) {
                return Err(ValidationError::UnsupportedInstruction {
                    inst,
                    opcode,
                    reason: format!("{} is not supported on riscv32", opcode),
                }
                .into());
            }
        }
    }

    Ok(())
}

// Placeholder validation functions - will be expanded in later phases
fn validate_iadd(_func: &Function, _inst: Inst, _data: &DataFlowGraph) -> CodegenResult<()> {
    Ok(())
}

fn validate_sdiv(_func: &Function, _inst: Inst, _data: &DataFlowGraph) -> CodegenResult<()> {
    Ok(())
}

fn validate_fadd(_func: &Function, _inst: Inst, _data: &DataFlowGraph) -> CodegenResult<()> {
    Ok(())
}

fn validate_udiv(func: &Function, inst: Inst, data: &DataFlowGraph) -> CodegenResult<()> {
    validate_div_rem_instruction(func, inst, data, Opcode::Udiv)
}

fn validate_urem(func: &Function, inst: Inst, data: &DataFlowGraph) -> CodegenResult<()> {
    validate_div_rem_instruction(func, inst, data, Opcode::Urem)
}

fn validate_srem(func: &Function, inst: Inst, data: &DataFlowGraph) -> CodegenResult<()> {
    validate_div_rem_instruction(func, inst, data, Opcode::Srem)
}

fn validate_div_rem_instruction(
    func: &Function,
    inst: Inst,
    _data: &DataFlowGraph,
    opcode: Opcode,
) -> CodegenResult<()> {
    // Get the type of the first argument (all args should have the same type)
    if let Some(&arg) = func.dfg.inst_args(inst).first() {
        let arg_ty = func.dfg.value_type(arg);

        // i64 division/remainder is not yet implemented on riscv32
        if arg_ty == I64 {
            return Err(ValidationError::UnsupportedCombination {
                inst,
                opcode,
                types: vec![arg_ty],
                reason: format!("i64 {} is not yet implemented on riscv32", opcode),
            }
            .into());
        }
    }

    Ok(())
}

fn validate_bswap(func: &Function, inst: Inst, _data: &DataFlowGraph) -> CodegenResult<()> {
    // Get the type of the first argument
    if let Some(&arg) = func.dfg.inst_args(inst).first() {
        let arg_ty = func.dfg.value_type(arg);

        // i64 bswap is not supported on riscv32
        if arg_ty == I64 {
            return Err(ValidationError::UnsupportedCombination {
                inst,
                opcode: Opcode::Bswap,
                types: vec![arg_ty],
                reason: "i64 bswap is not supported on riscv32".to_string(),
            }
            .into());
        }
    }

    Ok(())
}

fn validate_bitrev(func: &Function, inst: Inst, _data: &DataFlowGraph) -> CodegenResult<()> {
    // Get the type of the first argument
    if let Some(&arg) = func.dfg.inst_args(inst).first() {
        let arg_ty = func.dfg.value_type(arg);

        // i64 bitrev is not supported on riscv32
        if arg_ty == I64 {
            return Err(ValidationError::UnsupportedCombination {
                inst,
                opcode: Opcode::Bitrev,
                types: vec![arg_ty],
                reason: "i64 bitrev is not supported on riscv32".to_string(),
            }
            .into());
        }
    }

    Ok(())
}

fn validate_bmask(func: &Function, inst: Inst, _data: &DataFlowGraph) -> CodegenResult<()> {
    // bmask instruction is not supported on riscv32 (not needed for GLSL)
    return Err(ValidationError::UnsupportedInstruction {
        inst,
        opcode: Opcode::Bmask,
        reason: "bmask is not supported on riscv32 (not needed for GLSL)".to_string(),
    }
    .into());
}

fn validate_overflow_instruction(
    func: &Function,
    inst: Inst,
    opcode: Opcode,
    data: &DataFlowGraph,
) -> CodegenResult<()> {
    // Get result type for overflow instructions
    if let Some(result) = func.dfg.inst_results(inst).first() {
        let result_ty = func.dfg.value_type(*result);

        // Overflow instructions do not support i128 on riscv32
        if result_ty == I128 {
            return Err(super::error::ValidationError::UnsupportedType {
                ty: result_ty,
                context: format!("{} instruction", opcode),
            }
            .into());
        }

        // i64 overflow instructions are not supported on riscv32 (not needed for GLSL)
        if result_ty == I64 {
            return Err(ValidationError::UnsupportedCombination {
                inst,
                opcode,
                types: vec![result_ty],
                reason: format!(
                    "i64 {} is not supported on riscv32 (not needed for GLSL)",
                    opcode
                ),
            }
            .into());
        }
    }

    Ok(())
}

/// Check if a specific RISC-V extension is enabled on the backend
fn check_extension(backend: &super::super::Riscv32Backend, ext: supported::RiscvExtension) -> bool {
    match ext {
        supported::RiscvExtension::I => true, // Always required
        supported::RiscvExtension::M => backend.isa_flags.has_m(),
        supported::RiscvExtension::F => backend.isa_flags.has_f(),
        supported::RiscvExtension::D => backend.isa_flags.has_d(),
        supported::RiscvExtension::A => backend.isa_flags.has_a(),
        // C extension is split into sub-extensions; zca is the base compressed extension
        supported::RiscvExtension::C => backend.isa_flags.has_zca(),
        supported::RiscvExtension::Zba => backend.isa_flags.has_zba(),
        supported::RiscvExtension::Zbb => backend.isa_flags.has_zbb(),
        supported::RiscvExtension::Zbc => backend.isa_flags.has_zbc(),
        supported::RiscvExtension::Zbs => backend.isa_flags.has_zbs(),
        supported::RiscvExtension::Zca => backend.isa_flags.has_zca(),
        supported::RiscvExtension::Zcb => backend.isa_flags.has_zcb(),
        supported::RiscvExtension::Zcd => backend.isa_flags.has_zcd(),
        supported::RiscvExtension::Zcf => backend.isa_flags.has_zcf(),
        supported::RiscvExtension::Zfa => backend.isa_flags.has_zfa(),
        supported::RiscvExtension::Zfh => backend.isa_flags.has_zfh(),
        supported::RiscvExtension::Zfhmin => backend.isa_flags.has_zfhmin(),
        supported::RiscvExtension::Zicsr => backend.isa_flags.has_zicsr(),
        supported::RiscvExtension::Zifencei => backend.isa_flags.has_zifencei(),
        supported::RiscvExtension::Zicond => backend.isa_flags.has_zicond(),
        supported::RiscvExtension::Zbkb => backend.isa_flags.has_zbkb(),
        supported::RiscvExtension::Zbkc => backend.isa_flags.has_zbkc(),
        supported::RiscvExtension::Zbkx => backend.isa_flags.has_zbkx(),
        supported::RiscvExtension::Zkn => backend.isa_flags.has_zkn(),
        supported::RiscvExtension::Zks => backend.isa_flags.has_zks(),
        supported::RiscvExtension::V => backend.isa_flags.has_v(),
        supported::RiscvExtension::Zvfh => backend.isa_flags.has_zvfh(),
    }
}
