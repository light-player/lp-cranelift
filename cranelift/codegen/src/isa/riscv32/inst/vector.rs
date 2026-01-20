//! RISC-V 32-bit Vector Extension Support
//!
//! NOTE: Vector support deferred to Phase 2.
//! This file contains minimal type definitions to satisfy the build system.
//! Full vector instruction support will be added in a future phase.

use super::{Type, UImm5};
use crate::isa::riscv32::lower::isle::generated_code::{
    VecAMode, VecAluOpRImm5, VecAluOpRR, VecAluOpRRImm5, VecAluOpRRR, VecAluOpRRRImm5,
    VecAluOpRRRR, VecAvl, VecElementWidth, VecLmul, VecMaskMode, VecOpCategory, VecTailMode,
};
use crate::machinst::{OperandVisitor, Reg, RegClass};
use core::fmt;

/// Vector Opcode Masking
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VecOpMasking {
    Enabled { reg: Reg },
    Disabled,
}

// Minimal stub implementations for vector types

impl VecAvl {
    pub fn _static(size: u32) -> Self {
        VecAvl::Static {
            size: UImm5::maybe_from_u8(size as u8).expect("Invalid size for AVL"),
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            VecAvl::Static { .. } => true,
        }
    }

    pub fn unwrap_static(&self) -> UImm5 {
        match self {
            VecAvl::Static { size } => *size,
        }
    }
}

impl Copy for VecAvl {}

impl PartialEq for VecAvl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VecAvl::Static { size: lhs }, VecAvl::Static { size: rhs }) => lhs == rhs,
        }
    }
}

impl fmt::Display for VecAvl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VecAvl::Static { size } => write!(f, "{size}"),
        }
    }
}

impl VecElementWidth {
    pub fn from_type(ty: Type) -> Self {
        Self::from_bits(ty.lane_bits())
    }

    pub fn from_bits(bits: u32) -> Self {
        match bits {
            8 => VecElementWidth::E8,
            16 => VecElementWidth::E16,
            32 => VecElementWidth::E32,
            64 => VecElementWidth::E64,
            _ => panic!("Invalid number of bits for VecElementWidth: {bits}"),
        }
    }

    pub fn bits(&self) -> u32 {
        match self {
            VecElementWidth::E8 => 8,
            VecElementWidth::E16 => 16,
            VecElementWidth::E32 => 32,
            VecElementWidth::E64 => 64,
        }
    }

    pub fn encode(&self) -> u32 {
        match self {
            VecElementWidth::E8 => 0b000,
            VecElementWidth::E16 => 0b001,
            VecElementWidth::E32 => 0b010,
            VecElementWidth::E64 => 0b011,
        }
    }
}

impl fmt::Display for VecElementWidth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "e{}", self.bits())
    }
}

impl VecLmul {
    pub fn encode(&self) -> u32 {
        match self {
            VecLmul::LmulF8 => 0b101,
            VecLmul::LmulF4 => 0b110,
            VecLmul::LmulF2 => 0b111,
            VecLmul::Lmul1 => 0b000,
            VecLmul::Lmul2 => 0b001,
            VecLmul::Lmul4 => 0b010,
            VecLmul::Lmul8 => 0b011,
        }
    }
}

impl fmt::Display for VecLmul {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VecLmul::LmulF8 => write!(f, "mf8"),
            VecLmul::LmulF4 => write!(f, "mf4"),
            VecLmul::LmulF2 => write!(f, "mf2"),
            VecLmul::Lmul1 => write!(f, "m1"),
            VecLmul::Lmul2 => write!(f, "m2"),
            VecLmul::Lmul4 => write!(f, "m4"),
            VecLmul::Lmul8 => write!(f, "m8"),
        }
    }
}

impl VecTailMode {
    pub fn encode(&self) -> u32 {
        match self {
            VecTailMode::Agnostic => 1,
            VecTailMode::Undisturbed => 0,
        }
    }
}

impl fmt::Display for VecTailMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VecTailMode::Agnostic => write!(f, "ta"),
            VecTailMode::Undisturbed => write!(f, "tu"),
        }
    }
}

impl VecMaskMode {
    pub fn encode(&self) -> u32 {
        match self {
            VecMaskMode::Agnostic => 1,
            VecMaskMode::Undisturbed => 0,
        }
    }
}

impl fmt::Display for VecMaskMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VecMaskMode::Agnostic => write!(f, "ma"),
            VecMaskMode::Undisturbed => write!(f, "mu"),
        }
    }
}

/// VType - Vector Type Register
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct VType {
    pub sew: VecElementWidth,
    pub lmul: VecLmul,
    pub tail_mode: VecTailMode,
    pub mask_mode: VecMaskMode,
}

impl VType {
    pub fn encode(&self) -> u32 {
        let sew = self.sew.encode();
        let lmul = self.lmul.encode();
        let ta = self.tail_mode.encode();
        let ma = self.mask_mode.encode();

        (ma << 7) | (ta << 6) | (sew << 3) | lmul
    }
}

impl fmt::Display for VType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.sew, self.lmul, self.tail_mode, self.mask_mode
        )
    }
}

/// VState - Vector State
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct VState {
    pub avl: VecAvl,
    pub vtype: VType,
}

impl fmt::Display for VState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.avl,
            self.vtype.sew,
            self.vtype.lmul,
            if self.vtype.tail_mode == VecTailMode::Agnostic
                && self.vtype.mask_mode == VecMaskMode::Agnostic
            {
                "ma,ta"
            } else {
                "mu,tu"
            }
        )
    }
}

impl VState {
    pub fn from_type(ty: Type) -> Self {
        VState {
            avl: VecAvl::Static {
                size: UImm5::maybe_from_u8(ty.lane_count() as u8).unwrap(),
            },
            vtype: VType {
                sew: VecElementWidth::from_type(ty),
                lmul: VecLmul::Lmul1,
                tail_mode: VecTailMode::Agnostic,
                mask_mode: VecMaskMode::Agnostic,
            },
        }
    }
}

// Stubs for VecAMode methods (vector load/store addressing modes)
// Full implementation deferred to Phase 2
impl VecAMode {
    pub fn lumop(&self) -> u32 {
        unreachable!("Vector load/store support deferred to Phase 2")
    }

    pub fn sumop(&self) -> u32 {
        unreachable!("Vector load/store support deferred to Phase 2")
    }

    pub fn mop(&self) -> u32 {
        unreachable!("Vector load/store support deferred to Phase 2")
    }

    pub fn nf(&self) -> u32 {
        unreachable!("Vector load/store support deferred to Phase 2")
    }

    pub fn get_operands<V: OperandVisitor>(&mut self, _collector: &mut V) {
        unreachable!("Vector addressing mode support deferred to Phase 2")
    }
}

// Stubs for VecAluOp encoding methods
// Full implementation deferred to Phase 2
impl VecAluOpRRR {
    pub fn funct6(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn opcode(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn funct3(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn vs1_regclass(&mut self) -> RegClass {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn forbids_overlaps(&mut self, _mask: &VecOpMasking) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl fmt::Display for VecAluOpRRR {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl VecAluOpRRImm5 {
    pub fn funct6(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn opcode(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn funct3(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn imm_is_unsigned(&self) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn forbids_overlaps(&mut self, _mask: &VecOpMasking) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl fmt::Display for VecAluOpRRImm5 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl VecAluOpRRRImm5 {
    pub fn funct6(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn opcode(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn funct3(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn imm_is_unsigned(&self) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn forbids_overlaps(&mut self, _mask: &VecOpMasking) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl fmt::Display for VecAluOpRRRImm5 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl VecAluOpRR {
    pub fn funct6(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn opcode(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn funct3(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn vs_is_vs2_encoded(&self) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn aux_encoding(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn dst_regclass(&mut self) -> RegClass {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn src_regclass(&mut self) -> RegClass {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn forbids_overlaps(&mut self, _mask: &VecOpMasking) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl fmt::Display for VecAluOpRR {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl VecAluOpRImm5 {
    pub fn funct6(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn opcode(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn funct3(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn aux_encoding(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn forbids_overlaps(&mut self, _mask: &VecOpMasking) -> bool {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl fmt::Display for VecAluOpRImm5 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl VecAluOpRRRR {
    pub fn funct6(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn opcode(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn funct3(&self) -> u32 {
        unreachable!("Vector ALU support deferred to Phase 2")
    }

    pub fn vs1_regclass(&mut self) -> RegClass {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl fmt::Display for VecAluOpRRRR {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unreachable!("Vector ALU support deferred to Phase 2")
    }
}

impl VecOpMasking {
    pub fn encode(&self) -> u32 {
        unreachable!("Vector masking support deferred to Phase 2")
    }
}

impl fmt::Display for VecOpMasking {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VecOpMasking::Enabled { .. } => write!(f, ",v0.t"),
            VecOpMasking::Disabled => Ok(()),
        }
    }
}

impl VecOpCategory {
    pub fn encode(&self) -> u32 {
        unreachable!("Vector category support deferred to Phase 2")
    }
}
