use crate::ir::{Opcode, Type, types::*};
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;

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

/// Get the base extensions required for an opcode (excluding type-dependent extensions)
/// Returns Some(vec) if supported (vec may be empty for base ISA), None if unsupported
fn opcode_base_extensions(opcode: Opcode) -> Option<Vec<RiscvExtension>> {
    match opcode {
        // Arithmetic requiring M extension
        Opcode::Imul | Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem => {
            Some(vec![RiscvExtension::M])
        }

        // Atomic operations requiring A extension
        Opcode::AtomicRmw | Opcode::AtomicCas => Some(vec![RiscvExtension::A]),

        // Floating point requiring F extension (base requirement; D added based on types)
        Opcode::Fadd
        | Opcode::Fsub
        | Opcode::Fmul
        | Opcode::Fdiv
        | Opcode::Fmin
        | Opcode::Fmax
        | Opcode::Fabs
        | Opcode::Fneg
        | Opcode::Sqrt
        | Opcode::Fma => Some(vec![RiscvExtension::F]),

        // Bit manipulation requiring Zbb extension
        Opcode::Rotl | Opcode::Rotr | Opcode::Clz | Opcode::Ctz | Opcode::Popcnt => {
            Some(vec![RiscvExtension::Zbb])
        }

        // Address generation - base extensions checked in instruction validation
        Opcode::IaddImm => Some(vec![]),

        // Immediate arithmetic
        Opcode::IaddImm => Some(vec![]),
        Opcode::ImulImm => Some(vec![RiscvExtension::M]),

        // Immediate bitwise - base ISA
        Opcode::BandImm | Opcode::BorImm | Opcode::BxorImm => Some(vec![]),

        // Immediate shifts - base ISA
        Opcode::IshlImm | Opcode::UshrImm | Opcode::SshrImm => Some(vec![]),

        // Base instructions - no extensions required
        Opcode::Iadd
        | Opcode::Isub
        | Opcode::Band
        | Opcode::Bor
        | Opcode::Bxor
        | Opcode::Bnot
        | Opcode::Ishl
        | Opcode::Ushr
        | Opcode::Sshr
        | Opcode::Jump
        | Opcode::Brif
        | Opcode::Return
        | Opcode::Call
        | Opcode::Load
        | Opcode::Store
        | Opcode::StackLoad
        | Opcode::StackStore
        | Opcode::StackAddr
        | Opcode::Iconst
        | Opcode::F32const
        | Opcode::F64const
        | Opcode::Icmp
        | Opcode::Nop => Some(vec![]),

        // Check extension requirements map for less common opcodes
        _ => extension_requirements()
            .get(&opcode)
            .cloned()
            .map(Some)
            .unwrap_or(None),
    }
}

/// Check if an opcode is supported on riscv32
pub fn is_opcode_supported(opcode: Opcode) -> bool {
    opcode_base_extensions(opcode).is_some()
}

/// Get the required extensions for a CLIF opcode and its types
/// Returns a vector of extensions that must all be enabled
/// Empty vector means no extensions required (base ISA)
///
/// Note: Types are passed separately because some opcodes require different
/// extensions based on their operand types (e.g., f32 requires F, f64 requires D).
pub fn required_extensions(opcode: Opcode, types: &[Type]) -> Vec<RiscvExtension> {
    // Get base extensions for this opcode
    let mut exts = match opcode_base_extensions(opcode) {
        Some(base_exts) => base_exts,
        None => return vec![], // Unsupported opcode, return empty (shouldn't happen in practice)
    };

    // Special handling for floating point opcodes that need type-dependent extensions
    if matches!(
        opcode,
        Opcode::Fadd
            | Opcode::Fsub
            | Opcode::Fmul
            | Opcode::Fdiv
            | Opcode::Fmin
            | Opcode::Fmax
            | Opcode::Fabs
            | Opcode::Fneg
            | Opcode::Sqrt
            | Opcode::Fma
    ) {
        // Check types to determine if D extension is needed for f64 operations
        for ty in types {
            match *ty {
                F64 => {
                    if !exts.contains(&RiscvExtension::D) {
                        exts.push(RiscvExtension::D);
                    }
                    // F is already included from base extensions
                }
                F16 => {
                    // F16 requires Zfh or Zfhmin (check in type validation)
                    // For now, we'll validate this separately
                }
                _ => {}
            }
        }
    }

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

// Fallback map for opcodes not covered in the match statement above
// This is for less common opcodes or ones that need special handling
// Most opcodes should be handled in the match statement in required_extensions()
// Example:
// m.insert(Opcode::SomeRareOpcode, vec![RiscvExtension::Zbb]);
fn extension_requirements() -> HashMap<Opcode, Vec<RiscvExtension>> {
    let mut m = HashMap::new();

    m
}
