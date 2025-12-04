//! Structured RISC-V instruction representation.
//!
//! This module provides a structured representation of RISC-V instructions
//! as Rust enums, enabling type-safe pattern matching and testing.

use super::regs::Gpr;

/// A structured representation of a RISC-V instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inst {
    // Arithmetic instructions
    /// ADD: rd = rs1 + rs2
    Add { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// SUB: rd = rs1 - rs2
    Sub { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// MUL: rd = rs1 * rs2 (M extension)
    Mul { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// MULH: rd = high 32 bits of (rs1 * rs2) (signed, M extension)
    Mulh { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// DIV: rd = rs1 / rs2 (signed, M extension)
    Div { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// REM: rd = rs1 % rs2 (signed, M extension)
    Rem { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// ADDI: rd = rs1 + imm
    Addi { rd: Gpr, rs1: Gpr, imm: i32 },

    // Load/Store instructions
    /// LB: rd = sign_extend(mem[rs1 + imm][7:0])
    Lb { rd: Gpr, rs1: Gpr, imm: i32 },
    /// LH: rd = sign_extend(mem[rs1 + imm][15:0])
    Lh { rd: Gpr, rs1: Gpr, imm: i32 },
    /// LW: rd = mem[rs1 + imm]
    Lw { rd: Gpr, rs1: Gpr, imm: i32 },
    /// LBU: rd = zero_extend(mem[rs1 + imm][7:0])
    Lbu { rd: Gpr, rs1: Gpr, imm: i32 },
    /// LHU: rd = zero_extend(mem[rs1 + imm][15:0])
    Lhu { rd: Gpr, rs1: Gpr, imm: i32 },
    /// SB: mem[rs1 + imm][7:0] = rs2[7:0]
    Sb { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// SH: mem[rs1 + imm][15:0] = rs2[15:0]
    Sh { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// SW: mem[rs1 + imm] = rs2
    Sw { rs1: Gpr, rs2: Gpr, imm: i32 },

    // Control flow instructions
    /// JAL: rd = pc + 4; pc = pc + imm
    Jal { rd: Gpr, imm: i32 },
    /// JALR: rd = pc + 4; pc = rs1 + imm
    Jalr { rd: Gpr, rs1: Gpr, imm: i32 },
    /// BEQ: if rs1 == rs2, pc = pc + imm
    Beq { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// BNE: if rs1 != rs2, pc = pc + imm
    Bne { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// BLT: if rs1 < rs2 (signed), pc = pc + imm
    Blt { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// BGE: if rs1 >= rs2 (signed), pc = pc + imm
    Bge { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// BLTU: if rs1 < rs2 (unsigned), pc = pc + imm
    Bltu { rs1: Gpr, rs2: Gpr, imm: i32 },
    /// BGEU: if rs1 >= rs2 (unsigned), pc = pc + imm
    Bgeu { rs1: Gpr, rs2: Gpr, imm: i32 },

    // Comparison instructions
    /// SLT: rd = (rs1 < rs2) ? 1 : 0 (signed)
    Slt { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// SLTI: rd = (rs1 < imm) ? 1 : 0 (signed)
    Slti { rd: Gpr, rs1: Gpr, imm: i32 },
    /// SLTU: rd = (rs1 < rs2) ? 1 : 0 (unsigned)
    Sltu { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// SLTIU: rd = (rs1 < imm) ? 1 : 0 (unsigned)
    Sltiu { rd: Gpr, rs1: Gpr, imm: i32 },
    /// XORI: rd = rs1 ^ imm
    Xori { rd: Gpr, rs1: Gpr, imm: i32 },

    // Logical instructions
    /// AND: rd = rs1 & rs2
    And { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// ANDI: rd = rs1 & imm
    Andi { rd: Gpr, rs1: Gpr, imm: i32 },
    /// OR: rd = rs1 | rs2
    Or { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// ORI: rd = rs1 | imm
    Ori { rd: Gpr, rs1: Gpr, imm: i32 },
    /// XOR: rd = rs1 ^ rs2
    Xor { rd: Gpr, rs1: Gpr, rs2: Gpr },

    // Shift instructions
    /// SLL: rd = rs1 << rs2 (logical left shift)
    Sll { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// SLLI: rd = rs1 << imm (logical left shift immediate)
    Slli { rd: Gpr, rs1: Gpr, imm: i32 },
    /// SRL: rd = rs1 >> rs2 (logical right shift)
    Srl { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// SRLI: rd = rs1 >> imm (logical right shift immediate)
    Srli { rd: Gpr, rs1: Gpr, imm: i32 },
    /// SRA: rd = rs1 >> rs2 (arithmetic right shift)
    Sra { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// SRAI: rd = rs1 >> imm (arithmetic right shift immediate)
    Srai { rd: Gpr, rs1: Gpr, imm: i32 },

    // Immediate generation
    /// LUI: rd = imm << 12
    /// LUI: rd = imm
    /// `imm` is the sign-extended and shifted immediate value (already in final form)
    Lui { rd: Gpr, imm: i32 },
    /// AUIPC: rd = pc + imm
    /// `imm` is the sign-extended and shifted immediate value (already in final form)
    Auipc { rd: Gpr, imm: i32 },

    // System instructions
    /// ECALL: Environment call (syscall)
    Ecall,
    /// EBREAK: Environment break (halt/debug breakpoint)
    Ebreak,
    /// FENCE: Memory ordering (no-op in single-threaded emulator)
    Fence,

    // Atomic instructions (A extension)
    /// LR.W: Load reserved word
    LrW { rd: Gpr, rs1: Gpr },
    /// SC.W: Store conditional word
    ScW { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// AMOSWAP.W: Atomic swap word
    AmoswapW { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// AMOADD.W: Atomic add word
    AmoaddW { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// AMOXOR.W: Atomic XOR word
    AmoxorW { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// AMOAND.W: Atomic AND word
    AmoandW { rd: Gpr, rs1: Gpr, rs2: Gpr },
    /// AMOOR.W: Atomic OR word
    AmoorW { rd: Gpr, rs1: Gpr, rs2: Gpr },

    // Compressed instructions (RVC extension)
    // These expand to standard instruction forms

    /// C.ADDI: rd = rd + imm (expands to ADDI rd, rd, imm)
    CAddi { rd: Gpr, imm: i32 },
    /// C.LI: rd = imm (expands to ADDI rd, x0, imm)
    CLi { rd: Gpr, imm: i32 },
    /// C.LUI: rd = imm << 12 (expands to LUI)
    CLui { rd: Gpr, imm: i32 },
    /// C.MV: rd = rs (expands to ADD rd, x0, rs)
    CMv { rd: Gpr, rs: Gpr },
    /// C.ADD: rd = rd + rs (expands to ADD rd, rd, rs)
    CAdd { rd: Gpr, rs: Gpr },
    /// C.SUB: rd = rd - rs (expands to SUB rd, rd, rs)
    CSub { rd: Gpr, rs: Gpr },
    /// C.AND: rd = rd & rs (expands to AND rd, rd, rs)
    CAnd { rd: Gpr, rs: Gpr },
    /// C.OR: rd = rd | rs (expands to OR rd, rd, rs)
    COr { rd: Gpr, rs: Gpr },
    /// C.XOR: rd = rd ^ rs (expands to XOR rd, rd, rs)
    CXor { rd: Gpr, rs: Gpr },
    /// C.LW: rd = mem[rs + offset] (expands to LW)
    CLw { rd: Gpr, rs: Gpr, offset: i32 },
    /// C.SW: mem[rs1 + offset] = rs2 (expands to SW)
    CSw { rs1: Gpr, rs2: Gpr, offset: i32 },
    /// C.J: pc = pc + offset (expands to JAL x0, offset)
    CJ { offset: i32 },
    /// C.JR: pc = rs (expands to JALR x0, rs, 0)
    CJr { rs: Gpr },
    /// C.JALR: ra = pc + 2; pc = rs (expands to JALR x1, rs, 0)
    CJalr { rs: Gpr },
    /// C.BEQZ: if rs == 0, pc = pc + offset (expands to BEQ rs, x0, offset)
    CBeqz { rs: Gpr, offset: i32 },
    /// C.BNEZ: if rs != 0, pc = pc + offset (expands to BNE rs, x0, offset)
    CBnez { rs: Gpr, offset: i32 },
    /// C.SLLI: rd = rd << imm (expands to SLLI rd, rd, imm)
    CSlli { rd: Gpr, imm: i32 },
    /// C.SRLI: rd = rd >> imm (logical, expands to SRLI rd, rd, imm)
    CSrli { rd: Gpr, imm: i32 },
    /// C.SRAI: rd = rd >> imm (arithmetic, expands to SRAI rd, rd, imm)
    CSrai { rd: Gpr, imm: i32 },
    /// C.ANDI: rd = rd & imm (expands to ANDI rd, rd, imm)
    CAndi { rd: Gpr, imm: i32 },
    /// C.ADDI16SP: sp = sp + imm (expands to ADDI sp, sp, imm)
    CAddi16sp { imm: i32 },
    /// C.ADDI4SPN: rd = sp + imm (expands to ADDI rd, sp, imm)
    CAddi4spn { rd: Gpr, imm: i32 },
    /// C.LWSP: rd = mem[sp + offset] (expands to LW rd, sp, offset)
    CLwsp { rd: Gpr, offset: i32 },
    /// C.SWSP: mem[sp + offset] = rs (expands to SW sp, rs, offset)
    CSwsp { rs: Gpr, offset: i32 },
    /// C.JAL: ra = pc + 2; pc = pc + offset (RV32 only, expands to JAL x1, offset)
    CJal { offset: i32 },
    /// C.NOP: No operation (expands to ADDI x0, x0, 0)
    CNop,
    /// C.EBREAK: Breakpoint (same as EBREAK)
    CEbreak,
}

impl Inst {
    /// Encode this instruction to its binary representation.
    pub fn encode(&self) -> u32 {
        use super::encode::*;
        match self {
            Inst::Add { rd, rs1, rs2 } => add(*rd, *rs1, *rs2),
            Inst::Sub { rd, rs1, rs2 } => sub(*rd, *rs1, *rs2),
            Inst::Mul { rd, rs1, rs2 } => mul(*rd, *rs1, *rs2),
            Inst::Mulh { rd, rs1, rs2 } => mulh(*rd, *rs1, *rs2),
            Inst::Div { rd, rs1, rs2 } => div(*rd, *rs1, *rs2),
            Inst::Rem { rd, rs1, rs2 } => rem(*rd, *rs1, *rs2),
            Inst::Addi { rd, rs1, imm } => addi(*rd, *rs1, *imm),
            Inst::Lb { rd, rs1, imm } => lb(*rd, *rs1, *imm),
            Inst::Lh { rd, rs1, imm } => lh(*rd, *rs1, *imm),
            Inst::Lw { rd, rs1, imm } => lw(*rd, *rs1, *imm),
            Inst::Lbu { rd, rs1, imm } => lbu(*rd, *rs1, *imm),
            Inst::Lhu { rd, rs1, imm } => lhu(*rd, *rs1, *imm),
            Inst::Sb { rs1, rs2, imm } => sb(*rs1, *rs2, *imm),
            Inst::Sh { rs1, rs2, imm } => sh(*rs1, *rs2, *imm),
            Inst::Sw { rs1, rs2, imm } => sw(*rs1, *rs2, *imm),
            Inst::Jal { rd, imm } => jal(*rd, *imm),
            Inst::Jalr { rd, rs1, imm } => jalr(*rd, *rs1, *imm),
            Inst::Beq { rs1, rs2, imm } => beq(*rs1, *rs2, *imm),
            Inst::Bne { rs1, rs2, imm } => bne(*rs1, *rs2, *imm),
            Inst::Blt { rs1, rs2, imm } => blt(*rs1, *rs2, *imm),
            Inst::Bge { rs1, rs2, imm } => bge(*rs1, *rs2, *imm),
            Inst::Bltu { rs1, rs2, imm } => bltu(*rs1, *rs2, *imm),
            Inst::Bgeu { rs1, rs2, imm } => bgeu(*rs1, *rs2, *imm),
            Inst::Slt { rd, rs1, rs2 } => slt(*rd, *rs1, *rs2),
            Inst::Slti { rd, rs1, imm } => slti(*rd, *rs1, *imm),
            Inst::Sltu { rd, rs1, rs2 } => sltu(*rd, *rs1, *rs2),
            Inst::Sltiu { rd, rs1, imm } => sltiu(*rd, *rs1, *imm),
            Inst::Xori { rd, rs1, imm } => xori(*rd, *rs1, *imm),
            Inst::And { rd, rs1, rs2 } => and(*rd, *rs1, *rs2),
            Inst::Andi { rd, rs1, imm } => andi(*rd, *rs1, *imm),
            Inst::Or { rd, rs1, rs2 } => or(*rd, *rs1, *rs2),
            Inst::Ori { rd, rs1, imm } => ori(*rd, *rs1, *imm),
            Inst::Xor { rd, rs1, rs2 } => xor(*rd, *rs1, *rs2),
            Inst::Sll { rd, rs1, rs2 } => sll(*rd, *rs1, *rs2),
            Inst::Slli { rd, rs1, imm } => slli(*rd, *rs1, *imm),
            Inst::Srl { rd, rs1, rs2 } => srl(*rd, *rs1, *rs2),
            Inst::Srli { rd, rs1, imm } => srli(*rd, *rs1, *imm),
            Inst::Sra { rd, rs1, rs2 } => sra(*rd, *rs1, *rs2),
            Inst::Srai { rd, rs1, imm } => srai(*rd, *rs1, *imm),
            Inst::Lui { rd, imm } => lui(*rd, *imm),
            Inst::Auipc { rd, imm } => auipc(*rd, *imm),
            Inst::Ecall => ecall(),
            Inst::Ebreak => ebreak(),
            Inst::Fence => 0x0000000f, // fence (no-op encoding)

            // Atomic instructions - encode as placeholders (not typically needed)
            Inst::LrW { .. } => 0x1000202f, // lr.w a0, (zero)
            Inst::ScW { .. } => 0x1800202f, // sc.w a0, zero, (zero)
            Inst::AmoswapW { .. } => 0x0800202f, // amoswap.w a0, zero, (zero)
            Inst::AmoaddW { .. } => 0x0000202f, // amoadd.w a0, zero, (zero)
            Inst::AmoxorW { .. } => 0x2000202f, // amoxor.w a0, zero, (zero)
            Inst::AmoandW { .. } => 0x6000202f, // amoand.w a0, zero, (zero)
            Inst::AmoorW { .. } => 0x4000202f, // amoor.w a0, zero, (zero)

            // Compressed instructions - encode as their expanded forms
            Inst::CAddi { rd, imm } => addi(*rd, *rd, *imm),
            Inst::CLi { rd, imm } => addi(*rd, Gpr::Zero, *imm),
            Inst::CLui { rd, imm } => lui(*rd, *imm),
            Inst::CMv { rd, rs } => add(*rd, Gpr::Zero, *rs),
            Inst::CAdd { rd, rs } => add(*rd, *rd, *rs),
            Inst::CSub { rd, rs } => sub(*rd, *rd, *rs),
            Inst::CAnd { rd, rs } => and(*rd, *rd, *rs),
            Inst::COr { rd, rs } => or(*rd, *rd, *rs),
            Inst::CXor { rd, rs } => xor(*rd, *rd, *rs),
            Inst::CLw { rd, rs, offset } => lw(*rd, *rs, *offset),
            Inst::CSw { rs1, rs2, offset } => sw(*rs1, *rs2, *offset),
            Inst::CJ { offset } => jal(Gpr::Zero, *offset),
            Inst::CJr { rs } => jalr(Gpr::Zero, *rs, 0),
            Inst::CJalr { rs } => jalr(Gpr::Ra, *rs, 0),
            Inst::CBeqz { rs, offset } => beq(*rs, Gpr::Zero, *offset),
            Inst::CBnez { rs, offset } => bne(*rs, Gpr::Zero, *offset),
            Inst::CSlli { rd, imm } => slli(*rd, *rd, *imm),
            Inst::CSrli { rd, imm } => srli(*rd, *rd, *imm),
            Inst::CSrai { rd, imm } => srai(*rd, *rd, *imm),
            Inst::CAndi { rd, imm } => andi(*rd, *rd, *imm),
            Inst::CAddi16sp { imm } => addi(Gpr::Sp, Gpr::Sp, *imm),
            Inst::CAddi4spn { rd, imm } => addi(*rd, Gpr::Sp, *imm),
            Inst::CLwsp { rd, offset } => lw(*rd, Gpr::Sp, *offset),
            Inst::CSwsp { rs, offset } => sw(Gpr::Sp, *rs, *offset),
            Inst::CJal { offset } => jal(Gpr::Ra, *offset),
            Inst::CNop => addi(Gpr::Zero, Gpr::Zero, 0),
            Inst::CEbreak => ebreak(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::regs::Gpr, *};

    #[test]
    fn test_inst_encode_add() {
        let inst = Inst::Add {
            rd: Gpr::A0,
            rs1: Gpr::A1,
            rs2: Gpr::A2,
        };
        assert_eq!(inst.encode(), 0x00c58533);
    }

    #[test]
    fn test_inst_encode_addi() {
        let inst = Inst::Addi {
            rd: Gpr::A0,
            rs1: Gpr::A1,
            imm: 5,
        };
        assert_eq!(inst.encode(), 0x00558513);
    }

    #[test]
    fn test_inst_encode_addi_negative() {
        let inst = Inst::Addi {
            rd: Gpr::A0,
            rs1: Gpr::A1,
            imm: -5,
        };
        assert_eq!(inst.encode(), 0xffb58513);
    }

    #[test]
    fn test_inst_encode_lw() {
        let inst = Inst::Lw {
            rd: Gpr::A0,
            rs1: Gpr::A1,
            imm: 4,
        };
        assert_eq!(inst.encode(), 0x0045a503);
    }

    #[test]
    fn test_inst_encode_sw() {
        let inst = Inst::Sw {
            rs1: Gpr::A1,
            rs2: Gpr::A0,
            imm: 4,
        };
        assert_eq!(inst.encode(), 0x00a5a223);
    }

    #[test]
    fn test_inst_encode_jal() {
        let inst = Inst::Jal {
            rd: Gpr::Ra,
            imm: 0,
        };
        // jal ra, 0: opcode=0x6f, rd=1 (ra), imm=0
        // Encoding: 0x6f | (1 << 7) = 0x6f | 0x80 = 0xef
        assert_eq!(inst.encode(), 0x000000ef);
    }

    #[test]
    fn test_inst_encode_jalr() {
        let inst = Inst::Jalr {
            rd: Gpr::Zero,
            rs1: Gpr::Ra,
            imm: 0,
        };
        assert_eq!(inst.encode(), 0x00008067);
    }

    #[test]
    fn test_inst_encode_lui() {
        let inst = Inst::Lui {
            rd: Gpr::A0,
            imm: 0x12345000,
        };
        assert_eq!(inst.encode(), 0x12345537);
    }

    #[test]
    fn test_inst_encode_ecall() {
        let inst = Inst::Ecall;
        assert_eq!(inst.encode(), 0x00000073);
    }

    #[test]
    fn test_inst_encode_ebreak() {
        let inst = Inst::Ebreak;
        assert_eq!(inst.encode(), 0x00100073);
    }
}
