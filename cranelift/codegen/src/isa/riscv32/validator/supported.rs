use crate::ir::{Opcode, Type, types::*};
use hashbrown::HashMap;
use alloc::vec::Vec;
use alloc::vec;

/// RISC-V extension enum for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiscvExtension {
    I,        // Base integer instruction set (always required)
    M,        // Integer multiplication and division
    A,        // Atomic instructions
    F,        // Single-precision floating-point
    D,        // Double-precision floating-point (requires F)
    C,        // Compressed instructions (maps to Zca)
    Zba,      // Address generation
    Zbb,      // Basic bit-manipulation
    Zbc,      // Carry-less multiplication
    Zbs,      // Single-bit instructions
    Zca,      // Base compressed extension
    Zcb,      // Extra compressed instructions
    Zcd,      // Compressed double-precision FP loads/stores
    Zcf,      // Compressed single-precision FP loads/stores
    Zfa,      // Additional floating-point instructions
    Zfh,      // Half-precision floating-point (full)
    Zfhmin,   // Half-precision floating-point (minimal)
    Zicsr,    // Control and status register instructions
    Zifencei, // Instruction-fetch fence
    Zicond,   // Integer conditional operations
    Zbkb,     // Bit-manipulation for cryptography
    Zbkc,     // Carry-less multiplication for cryptography
    Zbkx,     // Crossbar permutations for cryptography
    Zkn,      // NIST Algorithm Suite
    Zks,      // ShangMi Algorithm Suite
    V,        // Vector extension
    Zvfh,     // Vector half-precision floating-point
}

impl RiscvExtension {
    pub fn name(&self) -> &'static str {
        match self {
            RiscvExtension::I => "I",
            RiscvExtension::M => "M",
            RiscvExtension::A => "A",
            RiscvExtension::F => "F",
            RiscvExtension::D => "D",
            RiscvExtension::C => "C",
            RiscvExtension::Zba => "Zba",
            RiscvExtension::Zbb => "Zbb",
            RiscvExtension::Zbc => "Zbc",
            RiscvExtension::Zbs => "Zbs",
            RiscvExtension::Zca => "Zca",
            RiscvExtension::Zcb => "Zcb",
            RiscvExtension::Zcd => "Zcd",
            RiscvExtension::Zcf => "Zcf",
            RiscvExtension::Zfa => "Zfa",
            RiscvExtension::Zfh => "Zfh",
            RiscvExtension::Zfhmin => "Zfhmin",
            RiscvExtension::Zicsr => "Zicsr",
            RiscvExtension::Zifencei => "Zifencei",
            RiscvExtension::Zicond => "Zicond",
            RiscvExtension::Zbkb => "Zbkb",
            RiscvExtension::Zbkc => "Zbkc",
            RiscvExtension::Zbkx => "Zbkx",
            RiscvExtension::Zkn => "Zkn",
            RiscvExtension::Zks => "Zks",
            RiscvExtension::V => "V",
            RiscvExtension::Zvfh => "Zvfh",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiscvExtension::I => "Base integer instruction set",
            RiscvExtension::M => "Integer multiplication and division",
            RiscvExtension::A => "Atomic instructions",
            RiscvExtension::F => "Single-precision floating-point (RV32F)",
            RiscvExtension::D => "Double-precision floating-point (RV32D, requires F)",
            RiscvExtension::C => "Compressed instructions (maps to Zca)",
            RiscvExtension::Zba => "Zba: Address Generation",
            RiscvExtension::Zbb => "Zbb: Basic bit-manipulation",
            RiscvExtension::Zbc => "Zbc: Carry-less multiplication",
            RiscvExtension::Zbs => "Zbs: Single-bit instructions",
            RiscvExtension::Zca => "Zca: Base compressed extension",
            RiscvExtension::Zcb => "Zcb: Extra compressed instructions",
            RiscvExtension::Zcd => "Zcd: Compressed double-precision FP loads/stores",
            RiscvExtension::Zcf => "Zcf: Compressed single-precision FP loads/stores",
            RiscvExtension::Zfa => "Zfa: Additional floating-point instructions",
            RiscvExtension::Zfh => "Zfh: Half-precision floating-point (full)",
            RiscvExtension::Zfhmin => "Zfhmin: Half-precision floating-point (minimal)",
            RiscvExtension::Zicsr => "Zicsr: Control and status register instructions",
            RiscvExtension::Zifencei => "Zifencei: Instruction-fetch fence",
            RiscvExtension::Zicond => "Zicond: Integer conditional operations",
            RiscvExtension::Zbkb => "Zbkb: Bit-manipulation for cryptography",
            RiscvExtension::Zbkc => "Zbkc: Carry-less multiplication for cryptography",
            RiscvExtension::Zbkx => "Zbkx: Crossbar permutations for cryptography",
            RiscvExtension::Zkn => "Zkn: NIST Algorithm Suite",
            RiscvExtension::Zks => "Zks: ShangMi Algorithm Suite",
            RiscvExtension::V => "V: Vector extension",
            RiscvExtension::Zvfh => "Zvfh: Vector half-precision floating-point",
        }
    }
}

/// Check if an opcode is supported on riscv32
pub fn is_opcode_supported(opcode: Opcode) -> bool {
    SUPPORTED_OPCODES.contains(&opcode)
}

/// Get the required extensions for a CLIF opcode and its types
/// Returns a vector of extensions that must all be enabled
/// Empty vector means no extensions required (base ISA)
///
/// Note: Types are passed separately because some opcodes require different
/// extensions based on their operand types (e.g., f32 requires F, f64 requires D).
pub fn required_extensions(opcode: Opcode, types: &[Type]) -> Vec<RiscvExtension> {
    // Use a match statement for all CLIF opcodes
    // This is extensible - add new opcodes and their requirements here
    let mut exts = match opcode {
        // Arithmetic requiring M extension
        Opcode::Imul | Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem => {
            vec![RiscvExtension::M]
        },

        // Atomic operations requiring A extension
        Opcode::AtomicRmw | Opcode::AtomicCas => vec![RiscvExtension::A],

        // Floating point requiring F/D extensions (type-dependent)
        Opcode::Fadd | Opcode::Fsub | Opcode::Fmul | Opcode::Fdiv
        | Opcode::Fmin | Opcode::Fmax | Opcode::Fabs | Opcode::Fneg
        | Opcode::Sqrt | Opcode::Fma => {
            // Check types to determine if F or D is needed
            let mut required = Vec::new();
            for ty in types {
                match *ty {
                    F32 => {
                        if !required.contains(&RiscvExtension::F) {
                            required.push(RiscvExtension::F);
                        }
                    }
                    F64 => {
                        if !required.contains(&RiscvExtension::D) {
                            required.push(RiscvExtension::D);
                        }
                        if !required.contains(&RiscvExtension::F) {
                            required.push(RiscvExtension::F); // D requires F
                        }
                    }
                    F16 => {
                        // F16 requires Zfh or Zfhmin (check in type validation)
                        // For now, we'll validate this separately
                    }
                    _ => {}
                }
            }
            // Default to F if no floating point types found (shouldn't happen, but be safe)
            if required.is_empty() {
                vec![RiscvExtension::F]
            } else {
                required
            }
        },

        // Bit manipulation requiring Zbb extension
        Opcode::Rotl | Opcode::Rotr | Opcode::Clz | Opcode::Ctz | Opcode::Popcnt => {
            vec![RiscvExtension::Zbb]
        },

        // Address generation requiring Zba extension
        // Note: IaddImm may use Zba patterns, but this requires instruction analysis
        // For now, we'll validate this in instruction-specific validation
        Opcode::IaddImm => vec![], // Will be checked in validate_iadd_imm

        // Base instructions - no extensions required
        Opcode::Iadd | Opcode::Isub | Opcode::Band | Opcode::Bor | Opcode::Bxor
        | Opcode::Bnot | Opcode::Ishl | Opcode::Ushr | Opcode::Sshr
        | Opcode::Jump | Opcode::Brif | Opcode::Return | Opcode::Call
        | Opcode::Load | Opcode::Store | Opcode::Nop => vec![],

        // Default: check if in extension requirements map (for less common opcodes)
        _ => extension_requirements().get(&opcode)
            .cloned()
            .unwrap_or_default(),
    };

    // Also check type-level extension requirements
    for ty in types {
        let type_exts = type_required_extensions(*ty);
        for ext in type_exts {
            if !exts.contains(&ext) {
                exts.push(ext);
            }
        }
    }

    exts
}

/// Get required extensions for a type (if any)
/// This checks if a type itself requires an extension (e.g., f32 requires F)
pub fn type_required_extensions(ty: Type) -> Vec<RiscvExtension> {
    match ty {
        F32 => vec![RiscvExtension::F],
        F64 => vec![RiscvExtension::D, RiscvExtension::F], // D requires F
        F16 => vec![RiscvExtension::Zfhmin], // Minimal support requires Zfhmin (full requires Zfh)
        _ => vec![],
    }
}

/// Check if a type is supported on riscv32 (ignoring extensions)
pub fn is_type_supported(ty: Type) -> bool {
    match ty {
        I8 | I16 | I32 => true,
        I64 => true, // Partially supported
        I128 => false,
        F32 | F64 | F16 => true, // If appropriate extension enabled
        F128 => false,
        _ => false,
    }
}

const SUPPORTED_OPCODES: &[Opcode] = &[
    // Control flow
    Opcode::Jump,
    Opcode::Brif,
    Opcode::BrTable,
    Opcode::Trap,
    Opcode::Return,
    Opcode::Call,
    // ... etc
];

// Fallback map for opcodes not covered in the match statement above
// This is for less common opcodes or ones that need special handling
// Most opcodes should be handled in the match statement in required_extensions()
// Example:
// m.insert(Opcode::SomeRareOpcode, vec![RiscvExtension::Zbb]);
fn extension_requirements() -> HashMap<Opcode, Vec<RiscvExtension>> {
    let mut m = HashMap::new();

    m
}
