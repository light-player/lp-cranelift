//! RISC-V 32-bit instruction decoding.
//!
//! This module provides functions to decode RISC-V 32-bit instructions
//! into their structured representation.
//!
//! This implementation uses format-specific decoding (following embive's approach)
//! to only extract the fields needed for each instruction type, avoiding
//! the inefficiency of extracting all immediate formats for every instruction.

use super::{format::*, inst::Inst, regs::Gpr};

/// Decode a 32-bit instruction word into a structured representation.
pub fn decode_instruction(inst: u32) -> Result<Inst, alloc::string::String> {
    use alloc::format;

    let opcode = (inst & 0x7f) as u8;

    match opcode {
        0x33 => {
            // R-type (arithmetic)
            let r = TypeR::from_riscv(inst);
            let rd = Gpr::new(r.rd);
            let rs1 = Gpr::new(r.rs1);
            let rs2 = Gpr::new(r.rs2);
            let funct3 = (r.func & 0x7) as u8;
            let funct7 = ((r.func >> 3) & 0x7f) as u8;
            match (funct3, funct7) {
                (0x0, 0x0) => Ok(Inst::Add { rd, rs1, rs2 }),
                (0x0, 0x20) => Ok(Inst::Sub { rd, rs1, rs2 }),
                (0x0, 0x01) => Ok(Inst::Mul { rd, rs1, rs2 }),
                (0x4, 0x01) => Ok(Inst::Div { rd, rs1, rs2 }),
                (0x6, 0x01) => Ok(Inst::Rem { rd, rs1, rs2 }),
                (0x2, 0x0) => Ok(Inst::Slt { rd, rs1, rs2 }),
                (0x3, 0x0) => Ok(Inst::Sltu { rd, rs1, rs2 }),
                (0x4, 0x0) => Ok(Inst::Xor { rd, rs1, rs2 }),
                (0x6, 0x0) => Ok(Inst::Or { rd, rs1, rs2 }),
                (0x7, 0x0) => Ok(Inst::And { rd, rs1, rs2 }),
                (0x1, 0x0) => Ok(Inst::Sll { rd, rs1, rs2 }),
                (0x5, 0x0) => Ok(Inst::Srl { rd, rs1, rs2 }),
                (0x5, 0x20) => Ok(Inst::Sra { rd, rs1, rs2 }),
                _ => Err(format!(
                    "Unknown R-type instruction: funct3=0x{:x}, funct7=0x{:x}",
                    funct3, funct7
                )),
            }
        }
        0x13 => {
            // I-type (immediate arithmetic/logical/shift)
            let i = TypeI::from_riscv(inst);
            let rd = Gpr::new(i.rd);
            let rs1 = Gpr::new(i.rs1);
            let funct3 = i.func;
            let imm_i = i.imm;
            let funct7 = ((inst >> 25) & 0x7f) as u8;
            match funct3 {
                0x0 => Ok(Inst::Addi {
                    rd,
                    rs1,
                    imm: imm_i,
                }),
                0x2 => Ok(Inst::Slti {
                    rd,
                    rs1,
                    imm: imm_i,
                }),
                0x3 => Ok(Inst::Sltiu {
                    rd,
                    rs1,
                    imm: imm_i,
                }),
                0x4 => Ok(Inst::Xori {
                    rd,
                    rs1,
                    imm: imm_i,
                }),
                0x6 => Ok(Inst::Ori {
                    rd,
                    rs1,
                    imm: imm_i,
                }),
                0x7 => Ok(Inst::Andi {
                    rd,
                    rs1,
                    imm: imm_i,
                }),
                0x1 => {
                    // SLLI
                    if funct7 == 0 {
                        Ok(Inst::Slli {
                            rd,
                            rs1,
                            imm: imm_i,
                        })
                    } else {
                        Err(format!(
                            "Unknown I-type shift instruction: funct3=0x{:x}, funct7=0x{:x}",
                            funct3, funct7
                        ))
                    }
                }
                0x5 => {
                    // SRLI/SRAI
                    if funct7 == 0 {
                        Ok(Inst::Srli {
                            rd,
                            rs1,
                            imm: imm_i,
                        })
                    } else if funct7 == 0x20 {
                        Ok(Inst::Srai {
                            rd,
                            rs1,
                            imm: imm_i,
                        })
                    } else {
                        Err(format!(
                            "Unknown I-type shift instruction: funct3=0x{:x}, funct7=0x{:x}",
                            funct3, funct7
                        ))
                    }
                }
                _ => Err(format!(
                    "Unknown I-type arithmetic instruction: funct3=0x{:x}",
                    funct3
                )),
            }
        }
        0x03 => {
            // I-type (load)
            let i = TypeI::from_riscv(inst);
            let rd = Gpr::new(i.rd);
            let rs1 = Gpr::new(i.rs1);
            match i.func {
                0x2 => Ok(Inst::Lw {
                    rd,
                    rs1,
                    imm: i.imm,
                }),
                _ => Err(format!("Unknown load instruction: funct3=0x{:x}", i.func)),
            }
        }
        0x23 => {
            // S-type (store)
            let s = TypeS::from_riscv(inst);
            let rs1 = Gpr::new(s.rs1);
            let rs2 = Gpr::new(s.rs2);
            match s.func {
                0x2 => Ok(Inst::Sw {
                    rs1,
                    rs2,
                    imm: s.imm,
                }),
                _ => Err(format!("Unknown store instruction: funct3=0x{:x}", s.func)),
            }
        }
        0x37 => {
            // U-type (lui)
            let u = TypeU::from_riscv(inst);
            let rd = Gpr::new(u.rd);
            Ok(Inst::Lui { rd, imm: u.imm })
        }
        0x17 => {
            // U-type (auipc)
            let u = TypeU::from_riscv(inst);
            let rd = Gpr::new(u.rd);
            Ok(Inst::Auipc { rd, imm: u.imm })
        }
        0x6f => {
            // J-type (jal)
            let j = TypeJ::from_riscv(inst);
            let rd = Gpr::new(j.rd);
            Ok(Inst::Jal { rd, imm: j.imm })
        }
        0x67 => {
            // I-type (jalr)
            let i = TypeI::from_riscv(inst);
            let rd = Gpr::new(i.rd);
            let rs1 = Gpr::new(i.rs1);
            match i.func {
                0x0 => Ok(Inst::Jalr {
                    rd,
                    rs1,
                    imm: i.imm,
                }),
                _ => Err(format!("Unknown jalr instruction: funct3=0x{:x}", i.func)),
            }
        }
        0x63 => {
            // B-type (branch)
            let b = TypeB::from_riscv(inst);
            let rs1 = Gpr::new(b.rs1);
            let rs2 = Gpr::new(b.rs2);
            match b.func {
                0x0 => Ok(Inst::Beq {
                    rs1,
                    rs2,
                    imm: b.imm,
                }),
                0x1 => Ok(Inst::Bne {
                    rs1,
                    rs2,
                    imm: b.imm,
                }),
                0x4 => Ok(Inst::Blt {
                    rs1,
                    rs2,
                    imm: b.imm,
                }),
                0x5 => Ok(Inst::Bge {
                    rs1,
                    rs2,
                    imm: b.imm,
                }),
                _ => Err(format!("Unknown branch instruction: funct3=0x{:x}", b.func)),
            }
        }
        0x73 => {
            // System instructions
            if inst == 0x00000073 {
                Ok(Inst::Ecall)
            } else if inst == 0x00100073 {
                Ok(Inst::Ebreak)
            } else {
                Err(format!("Unknown system instruction: 0x{:08x}", inst))
            }
        }
        _ => Err(format!("Unknown opcode: 0x{:02x}", opcode)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::riscv32::{encode::auipc, inst::Inst};

    /// Test that encoding and decoding AUIPC produces consistent results
    fn test_auipc_round_trip(imm: i32, rd: Gpr) {
        // Encode
        let encoded = auipc(rd, imm);

        // Decode
        let decoded = decode_instruction(encoded).expect("Failed to decode");

        // Verify it's an AUIPC instruction
        match decoded {
            Inst::Auipc {
                rd: decoded_rd,
                imm: decoded_imm,
            } => {
                assert_eq!(
                    decoded_rd, rd,
                    "Register mismatch: expected {:?}, got {:?}",
                    rd, decoded_rd
                );

                // The decoded imm is the final i32 value (sign-extended and shifted)
                // It should match the encoded imm
                assert_eq!(
                    decoded_imm, imm,
                    "Immediate mismatch: encoded imm={} (0x{:08x}), decoded imm={} (0x{:08x})",
                    imm, imm as u32, decoded_imm, decoded_imm as u32
                );
            }
            _ => panic!("Expected AUIPC instruction, got {:?}", decoded),
        }
    }

    #[test]
    fn test_auipc_round_trip_zero() {
        test_auipc_round_trip(0, Gpr::T0);
    }

    #[test]
    fn test_auipc_round_trip_positive_small() {
        test_auipc_round_trip(0x1000, Gpr::T0); // 1 << 12
        test_auipc_round_trip(0x123000, Gpr::T0); // 0x123 << 12
        test_auipc_round_trip(0x1234000, Gpr::T0); // 0x1234 << 12
    }

    #[test]
    fn test_auipc_round_trip_positive_large() {
        test_auipc_round_trip(0x12345000, Gpr::T0); // 0x12345 << 12
        test_auipc_round_trip(0x7ffff000, Gpr::T0); // Max positive 20-bit << 12
    }

    #[test]
    fn test_auipc_round_trip_negative() {
        // Negative values: upper 20 bits with sign bit set, already shifted
        test_auipc_round_trip(0x80000000u32 as i32, Gpr::T0); // Min negative 20-bit << 12
        test_auipc_round_trip(0xfffff000u32 as i32, Gpr::T0); // -1 in 20-bit << 12
        test_auipc_round_trip(0xff000000u32 as i32, Gpr::T0); // Specific negative value << 12
    }

    #[test]
    fn test_auipc_round_trip_all_registers() {
        test_auipc_round_trip(0x12345000, Gpr::Zero);
        test_auipc_round_trip(0x12345000, Gpr::Ra);
        test_auipc_round_trip(0x12345000, Gpr::Sp);
        test_auipc_round_trip(0x12345000, Gpr::Gp);
        test_auipc_round_trip(0x12345000, Gpr::Tp);
        test_auipc_round_trip(0x12345000, Gpr::T0);
        test_auipc_round_trip(0x12345000, Gpr::T1);
        test_auipc_round_trip(0x12345000, Gpr::T2);
        test_auipc_round_trip(0x12345000, Gpr::S0);
        test_auipc_round_trip(0x12345000, Gpr::S1);
        test_auipc_round_trip(0x12345000, Gpr::A0);
        test_auipc_round_trip(0x12345000, Gpr::A1);
    }

    #[test]
    fn test_auipc_decode_specific_instructions() {
        // Test decoding specific instruction words
        // auipc t0, 0
        let inst = decode_instruction(0x00000297).expect("Failed to decode");
        match inst {
            Inst::Auipc { rd, imm } => {
                assert_eq!(rd, Gpr::T0);
                assert_eq!(imm, 0x00000);
            }
            _ => panic!("Expected AUIPC, got {:?}", inst),
        }

        // auipc t0, 0xff000 (upper 20 bits)
        // When decoded, this becomes 0xff000000 (sign-extended and shifted)
        let inst = decode_instruction(0xff000297).expect("Failed to decode");
        match inst {
            Inst::Auipc { rd, imm } => {
                assert_eq!(rd, Gpr::T0);
                assert_eq!(imm, 0xff000000u32 as i32);
            }
            _ => panic!("Expected AUIPC, got {:?}", inst),
        }

        // auipc t0, 0xfffff (upper 20 bits)
        // When decoded, this becomes 0xfffff000 (sign-extended and shifted)
        let inst = decode_instruction(0xfffff297).expect("Failed to decode");
        match inst {
            Inst::Auipc { rd, imm } => {
                assert_eq!(rd, Gpr::T0);
                assert_eq!(imm, 0xfffff000u32 as i32);
            }
            _ => panic!("Expected AUIPC, got {:?}", inst),
        }
    }

    #[test]
    fn test_auipc_sign_extension_behavior() {
        // Test that negative immediates are handled correctly
        // When we encode 0xfffff000 (the final value), it extracts 0xfffff as the upper 20 bits
        // When decoded, it should be 0xfffff000 (sign-extended and shifted)
        let encoded = auipc(Gpr::T0, 0xfffff000u32 as i32);
        let decoded = decode_instruction(encoded).expect("Failed to decode");
        match decoded {
            Inst::Auipc { imm, .. } => {
                // The decoded imm should be the sign-extended and shifted value 0xfffff000
                assert_eq!(imm, 0xfffff000u32 as i32);
            }
            _ => panic!("Expected AUIPC"),
        }

        // Same for 0xff000
        let encoded = auipc(Gpr::T0, 0xff000);
        let decoded = decode_instruction(encoded).expect("Failed to decode");
        match decoded {
            Inst::Auipc { imm, .. } => {
                assert_eq!(imm, 0xff000);
            }
            _ => panic!("Expected AUIPC"),
        }
    }
}
