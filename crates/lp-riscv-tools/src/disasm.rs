//! RISC-V 32-bit instruction disassembly.

use alloc::{collections::BTreeMap, format, string::String};

/// Disassemble a single RISC-V 32-bit instruction.
///
/// Returns a human-readable string like "add a0, a1, a2" or "jal ra, 16".
pub fn disassemble_instruction(inst: u32) -> String {
    disassemble_instruction_with_labels(inst, 0, None)
}

/// Disassemble a single RISC-V 32-bit instruction with label support.
///
/// # Arguments
///
/// * `inst` - The 32-bit instruction word
/// * `pc` - Program counter (address) of this instruction
/// * `labels` - Optional map of address -> label name, and reverse map for target lookups
///
/// Returns a human-readable string with labels substituted for offsets when available.
pub fn disassemble_instruction_with_labels(
    inst: u32,
    pc: u32,
    labels: Option<(&BTreeMap<u32, String>, &BTreeMap<u32, String>)>,
) -> String {
    use super::format::*;

    let opcode = inst & 0x7f;

    match opcode {
        0x33 => {
            // R-type (arithmetic)
            let r = TypeR::from_riscv(inst);
            let rd = r.rd;
            let rs1 = r.rs1;
            let rs2 = r.rs2;
            let funct3 = (r.func & 0x7) as u8;
            let funct7 = ((r.func >> 3) & 0x7f) as u8;
            match (funct3, funct7) {
                (0x0, 0x0) => format!("add {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2)),
                (0x0, 0x20) => {
                    format!("sub {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x0, 0x01) => {
                    format!("mul {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x4, 0x01) => {
                    format!("div {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x6, 0x01) => {
                    format!("rem {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x2, 0x0) => {
                    format!("slt {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x3, 0x0) => {
                    format!(
                        "sltu {}, {}, {}",
                        gpr_name(rd),
                        gpr_name(rs1),
                        gpr_name(rs2)
                    )
                }
                (0x4, 0x0) => {
                    format!("xor {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x6, 0x0) => {
                    format!("or {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x7, 0x0) => {
                    format!("and {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x1, 0x0) => {
                    format!("sll {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x5, 0x0) => {
                    format!("srl {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                (0x5, 0x20) => {
                    format!("sra {}, {}, {}", gpr_name(rd), gpr_name(rs1), gpr_name(rs2))
                }
                _ => format!("unknown_r_type 0x{:08x}", inst),
            }
        }
        0x13 => {
            // I-type (immediate arithmetic/logical/shift)
            let i = TypeI::from_riscv(inst);
            let rd = i.rd;
            let rs1 = i.rs1;
            let funct3 = i.func;
            let imm_i = i.imm;
            let imm_5bit = (imm_i & 0x1f) as u8;
            match funct3 {
                0x0 => format!("addi {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_i),
                0x2 => format!("slti {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_i),
                0x3 => format!("sltiu {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_i),
                0x4 => format!("xori {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_i),
                0x6 => format!("ori {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_i),
                0x7 => format!("andi {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_i),
                0x1 => {
                    // SLLI - check funct7 (bit 30) to distinguish from custom instructions
                    let funct7 = ((inst >> 25) & 0x7f) as u8;
                    if funct7 == 0 {
                        format!("slli {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_5bit)
                    } else {
                        format!("unknown_i_type 0x{:08x}", inst)
                    }
                }
                0x5 => {
                    // SRLI/SRAI - check funct7 (bit 30)
                    let funct7 = ((inst >> 25) & 0x7f) as u8;
                    if funct7 == 0 {
                        format!("srli {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_5bit)
                    } else if funct7 == 0x20 {
                        format!("srai {}, {}, {}", gpr_name(rd), gpr_name(rs1), imm_5bit)
                    } else {
                        format!("unknown_i_type 0x{:08x}", inst)
                    }
                }
                _ => format!("unknown_i_type 0x{:08x}", inst),
            }
        }
        0x03 => {
            // I-type (load)
            let i = TypeI::from_riscv(inst);
            let rd = i.rd;
            let rs1 = i.rs1;
            let funct3 = i.func;
            let imm_i = i.imm;
            match funct3 {
                0x2 => format!("lw {}, {}({})", gpr_name(rd), imm_i, gpr_name(rs1)),
                _ => format!("unknown_load 0x{:08x}", inst),
            }
        }
        0x23 => {
            // S-type (store)
            let s = TypeS::from_riscv(inst);
            let rs1 = s.rs1;
            let rs2 = s.rs2;
            let funct3 = s.func;
            let imm_s = s.imm;
            match funct3 {
                0x2 => format!("sw {}, {}({})", gpr_name(rs2), imm_s, gpr_name(rs1)),
                _ => format!("unknown_store 0x{:08x}", inst),
            }
        }
        0x37 => {
            // U-type (lui)
            let u = TypeU::from_riscv(inst);
            let rd = u.rd;
            let imm_u = (u.imm as u32) >> 12;
            format!("lui {}, 0x{:05x}", gpr_name(rd), imm_u)
        }
        0x17 => {
            // U-type (auipc)
            let u = TypeU::from_riscv(inst);
            let rd = u.rd;
            let imm_u = (u.imm as u32) >> 12;
            format!("auipc {}, 0x{:05x}", gpr_name(rd), imm_u)
        }
        0x6f => {
            // J-type (jal)
            let j = TypeJ::from_riscv(inst);
            let rd = j.rd;
            let imm_j = j.imm;
            let target = pc.wrapping_add(imm_j as u32);
            let label = if let Some((_, rev)) = labels {
                if let Some(name) = rev.get(&target) {
                    name.clone()
                } else {
                    format!("label_{}", target / 4)
                }
            } else {
                format!("label_{}", target / 4)
            };
            format!("jal {}, {}", gpr_name(rd), label)
        }
        0x67 => {
            // I-type (jalr)
            let i = TypeI::from_riscv(inst);
            let rd = i.rd;
            let rs1 = i.rs1;
            let funct3 = i.func;
            let imm_i = i.imm;
            match funct3 {
                0x0 => {
                    // For jalr, we can't determine the target statically, so just use the immediate
                    format!("jalr {}, {}({})", gpr_name(rd), imm_i, gpr_name(rs1))
                }
                _ => format!("unknown_jalr 0x{:08x}", inst),
            }
        }
        0x63 => {
            // B-type (branch)
            let b = TypeB::from_riscv(inst);
            let rs1 = b.rs1;
            let rs2 = b.rs2;
            let funct3 = b.func;
            let imm_b = b.imm;
            let target = pc.wrapping_add(imm_b as u32);
            let label = if let Some((_, rev)) = labels {
                if let Some(name) = rev.get(&target) {
                    name.clone()
                } else {
                    format!("label_{}", target / 4)
                }
            } else {
                format!("label_{}", target / 4)
            };
            match funct3 {
                0x0 => format!("beq {}, {}, {}", gpr_name(rs1), gpr_name(rs2), label),
                0x1 => format!("bne {}, {}, {}", gpr_name(rs1), gpr_name(rs2), label),
                0x4 => format!("blt {}, {}, {}", gpr_name(rs1), gpr_name(rs2), label),
                0x5 => format!("bge {}, {}, {}", gpr_name(rs1), gpr_name(rs2), label),
                _ => format!("unknown_branch 0x{:08x}", inst),
            }
        }
        0x73 => {
            // System instructions
            if inst == 0x00000073 {
                String::from("ecall")
            } else if inst == 0x00100073 {
                String::from("ebreak")
            } else {
                format!("unknown_system 0x{:08x}", inst)
            }
        }
        _ => format!("unknown 0x{:08x} (opcode=0x{:02x})", inst, opcode),
    }
}

/// Disassemble a code buffer containing RISC-V instructions.
///
/// Returns a formatted string with one instruction per line, showing
/// the address/offset and the disassembled instruction.
pub fn disassemble_code(code: &[u8]) -> String {
    disassemble_code_with_labels(code, None, true)
}

/// Disassemble a code buffer containing RISC-V instructions with label support.
///
/// # Arguments
///
/// * `code` - Binary code buffer to disassemble
/// * `labels` - Optional map of address -> label name
/// * `include_addresses` - Whether to include address prefixes (0x0000:) in output
///
/// Returns a formatted string with labels printed at their addresses and
/// used in branch/jump instructions. Auto-generates indexed labels for
/// branch/jump targets if not provided.
///
/// # Zero-Run Detection
///
/// Runs of 16 or more consecutive zero bytes (4-byte aligned) are automatically
/// detected and collapsed into a comment line showing the address range and byte count.
/// A `.org` directive is output after the comment to maintain address continuity,
/// making the disassembly reassemblable. The format is:
/// `; ... N zero bytes (0xSTART-0xEND) ...`
/// `.org 0xEND`
pub fn disassemble_code_with_labels(
    code: &[u8],
    labels: Option<&BTreeMap<u32, String>>,
    include_addresses: bool,
) -> String {
    let mut result = String::new();
    let mut offset = 0;

    // Build reverse map (address -> label) for efficient lookups
    let label_map = labels.map(|map| {
        let mut rev_map = BTreeMap::new();
        for (addr, name) in map.iter() {
            rev_map.insert(*addr, name.clone());
        }
        rev_map
    });

    // Collect all branch/jump targets to auto-generate labels
    let mut auto_labels = BTreeMap::new();
    let mut label_counter = 0;

    // First pass: identify all branch/jump targets
    let mut temp_offset = 0;
    while temp_offset + 4 <= code.len() {
        let inst_bytes = [
            code[temp_offset],
            code[temp_offset + 1],
            code[temp_offset + 2],
            code[temp_offset + 3],
        ];
        let inst = u32::from_le_bytes(inst_bytes);

        // Extract target addresses for branches and jumps
        let opcode = inst & 0x7f;
        match opcode {
            0x6f => {
                // JAL
                let j = super::format::TypeJ::from_riscv(inst);
                let target = (temp_offset as u32).wrapping_add(j.imm as u32);
                if label_map
                    .as_ref()
                    .map_or(true, |m| !m.contains_key(&target))
                {
                    auto_labels.entry(target).or_insert_with(|| {
                        label_counter += 1;
                        format!("label_{}", label_counter - 1)
                    });
                }
            }
            0x63 => {
                // Branch
                let b = super::format::TypeB::from_riscv(inst);
                let target = (temp_offset as u32).wrapping_add(b.imm as u32);
                if label_map
                    .as_ref()
                    .map_or(true, |m| !m.contains_key(&target))
                {
                    auto_labels.entry(target).or_insert_with(|| {
                        label_counter += 1;
                        format!("label_{}", label_counter - 1)
                    });
                }
            }
            _ => {}
        }

        temp_offset += 4;
    }

    // Merge provided labels with auto-generated ones
    let mut all_labels = BTreeMap::new();
    if let Some(provided) = labels {
        for (addr, name) in provided.iter() {
            all_labels.insert(*addr, name.clone());
        }
    }
    for (addr, name) in auto_labels.iter() {
        all_labels.entry(*addr).or_insert_with(|| name.clone());
    }

    // Build reverse map for instruction disassembly
    let rev_map = all_labels.clone();

    // Second pass: disassemble with labels
    while offset + 4 <= code.len() {
        let offset_u32 = offset as u32;
        // Check if there's a label at this address
        if let Some(label_name) = all_labels.get(&offset_u32) {
            result.push_str(&format!("{}:\n", label_name));
        }

        // Check for zero run (minimum 16 bytes = 4 instructions)
        if let Some(zero_len) = detect_zero_run(code, offset, 16) {
            let end_addr = offset + zero_len;

            // Format comment
            if include_addresses {
                result.push_str(&format!(
                    "; ... {} zero bytes (0x{:04x}-0x{:04x}) ...\n",
                    zero_len, offset, end_addr
                ));
            } else {
                result.push_str(&format!("; ... {} zero bytes ...\n", zero_len));
            }

            // Output .org directive to maintain address continuity
            // This makes the disassembly reassemblable
            result.push_str(&format!(".org 0x{:04x}\n", end_addr));

            // Skip the zero region
            offset = end_addr;
            continue;
        }

        // Read 32-bit instruction (little-endian)
        let inst_bytes = [
            code[offset],
            code[offset + 1],
            code[offset + 2],
            code[offset + 3],
        ];
        let inst = u32::from_le_bytes(inst_bytes);

        let labels_for_inst = if labels.is_some() || !rev_map.is_empty() {
            Some((&all_labels, &rev_map))
        } else {
            None
        };
        let disasm = disassemble_instruction_with_labels(inst, offset_u32, labels_for_inst);
        if include_addresses {
            result.push_str(&format!("0x{:04x}: {}\n", offset, disasm));
        } else {
            result.push_str(&format!("{}\n", disasm));
        }

        offset += 4;
    }

    // Handle remaining bytes (if any)
    if offset < code.len() {
        let offset_u32 = offset as u32;
        if let Some(label_name) = all_labels.get(&offset_u32) {
            result.push_str(&format!("{}:\n", label_name));
        }
        if include_addresses {
            result.push_str(&format!("0x{:04x}: <incomplete instruction>\n", offset));
        } else {
            result.push_str("<incomplete instruction>\n");
        }
    }

    result
}

/// Detect a run of zero bytes starting at the given offset.
///
/// Returns the length of the zero run if it's at least `min_bytes` long,
/// or `None` if no such run exists. Only detects runs that are 4-byte aligned.
fn detect_zero_run(code: &[u8], start: usize, min_bytes: usize) -> Option<usize> {
    // Check alignment
    if start % 4 != 0 {
        return None;
    }

    let mut len = 0;
    let mut pos = start;

    // Count consecutive zero bytes
    while pos < code.len() && code[pos] == 0 {
        len += 1;
        pos += 1;
    }

    // Only return if we found at least min_bytes and it's 4-byte aligned
    if len >= min_bytes && len % 4 == 0 {
        Some(len)
    } else {
        None
    }
}

/// Get the name of a general-purpose register.
fn gpr_name(num: u8) -> &'static str {
    match num {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0",
        11 => "a1",
        12 => "a2",
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{encode::*, regs::Gpr},
        *,
    };

    #[test]
    fn test_disassemble_add() {
        let inst = add(Gpr::A0, Gpr::A1, Gpr::A2);
        let disasm = disassemble_instruction(inst);
        assert_eq!(disasm, "add a0, a1, a2");
    }

    #[test]
    fn test_disassemble_sub() {
        let inst = sub(Gpr::A0, Gpr::A1, Gpr::A2);
        let disasm = disassemble_instruction(inst);
        assert_eq!(disasm, "sub a0, a1, a2");
    }

    #[test]
    fn test_disassemble_addi() {
        let inst = addi(Gpr::A0, Gpr::A1, 5);
        let disasm = disassemble_instruction(inst);
        assert_eq!(disasm, "addi a0, a1, 5");
    }

    #[test]
    fn test_disassemble_addi_negative() {
        let inst = addi(Gpr::A0, Gpr::A1, -5);
        let disasm = disassemble_instruction(inst);
        assert_eq!(disasm, "addi a0, a1, -5");
    }

    #[test]
    fn test_disassemble_lui() {
        let inst = lui(Gpr::A0, 0x12345000);
        let disasm = disassemble_instruction(inst);
        assert!(disasm.contains("lui a0"));
        assert!(disasm.contains("0x12345"));
    }

    #[test]
    fn test_disassemble_ecall() {
        let inst = ecall();
        let disasm = disassemble_instruction(inst);
        assert_eq!(disasm, "ecall");
    }

    #[test]
    fn test_disassemble_code() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&add(Gpr::A0, Gpr::A1, Gpr::A2).to_le_bytes());
        code.extend_from_slice(&addi(Gpr::A1, Gpr::A0, 10).to_le_bytes());
        code.extend_from_slice(&ecall().to_le_bytes());

        let disasm = disassemble_code(&code);
        assert!(disasm.contains("add a0, a1, a2"));
        assert!(disasm.contains("addi a1, a0, 10"));
        assert!(disasm.contains("ecall"));
    }

    #[test]
    fn test_disassemble_code_with_labels() {
        use alloc::{collections::BTreeMap, string::ToString, vec::Vec};

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());
        code.extend_from_slice(&beq(Gpr::A0, Gpr::A1, 8).to_le_bytes());
        code.extend_from_slice(&addi(Gpr::A0, Gpr::A0, 1).to_le_bytes());

        let labels = BTreeMap::from([(0x0008, "loop".to_string())]);
        let disasm = disassemble_code_with_labels(&code, Some(&labels), true);

        assert!(disasm.contains("loop:"));
        assert!(disasm.contains("beq"));
    }

    #[test]
    fn test_disassemble_code_auto_labels() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        code.extend_from_slice(&jal(Gpr::Ra, 8).to_le_bytes());
        code.extend_from_slice(&addi(Gpr::A0, Gpr::A0, 1).to_le_bytes());

        let disasm = disassemble_code_with_labels(&code, None, true);
        // Should auto-generate label_2 for the jal target
        assert!(disasm.contains("label_"));
        assert!(disasm.contains("jal"));
    }

    #[test]
    fn test_round_trip_assemble_disassemble() {
        use super::super::asm_parser::assemble_code;

        let asm = "addi a0, zero, 5\naddi a1, zero, 10\nadd a0, a0, a1\nebreak";
        let code = assemble_code(asm, None).unwrap();
        let disasm = disassemble_code(&code);

        // Check that all instructions are present
        assert!(disasm.contains("addi a0, zero, 5"));
        assert!(disasm.contains("addi a1, zero, 10"));
        assert!(disasm.contains("add a0, a0, a1"));
        assert!(disasm.contains("ebreak"));
    }

    #[test]
    fn test_round_trip_with_labels() {
        use alloc::{collections::BTreeMap, string::ToString};

        use super::super::asm_parser::assemble_code;

        let asm = "addi a0, zero, 5\nloop:\naddi a0, a0, 1\nbeq a0, a1, loop";
        let code = assemble_code(asm, None).unwrap();

        // Disassemble with labels
        let labels = BTreeMap::from([(0x0004, "loop".to_string())]);
        let disasm = disassemble_code_with_labels(&code, Some(&labels), true);

        assert!(disasm.contains("loop:"));
        assert!(disasm.contains("beq"));
    }

    #[test]
    fn test_disassemble_zero_run() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // Add 32 bytes of zeros
        code.extend_from_slice(&[0u8; 32]);
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());

        let disasm = disassemble_code(&code);
        assert!(disasm.contains("addi a0, zero, 5"));
        assert!(disasm.contains("zero bytes"));
        assert!(disasm.contains("addi a1, zero, 10"));
        assert!(!disasm.contains("add zero, zero, zero")); // Should not show NOPs
        assert!(disasm.contains(".org")); // Should include .org directive
    }

    #[test]
    fn test_disassemble_zero_run_at_start() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        // Start with 24 bytes of zeros
        code.extend_from_slice(&[0u8; 24]);
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());

        let disasm = disassemble_code(&code);
        assert!(disasm.contains("zero bytes"));
        assert!(disasm.contains("addi a0, zero, 5"));
        assert!(disasm.contains(".org 0x0018")); // Should have .org after zero run
    }

    #[test]
    fn test_disassemble_zero_run_in_middle() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // Add 20 bytes of zeros
        code.extend_from_slice(&[0u8; 20]);
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());

        let disasm = disassemble_code(&code);
        assert!(disasm.contains("addi a0, zero, 5"));
        assert!(disasm.contains("zero bytes"));
        assert!(disasm.contains("addi a1, zero, 10"));
    }

    #[test]
    fn test_disassemble_zero_run_at_end() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // Add 16 bytes of zeros at the end
        code.extend_from_slice(&[0u8; 16]);

        let disasm = disassemble_code(&code);
        assert!(disasm.contains("addi a0, zero, 5"));
        assert!(disasm.contains("zero bytes"));
    }

    #[test]
    fn test_disassemble_short_zero_run() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // Add 12 bytes of zeros (below 16-byte threshold)
        // Note: 0x00000000 is not a valid RISC-V instruction (opcode=0x00 is invalid)
        // So it will be disassembled as "unknown"
        code.extend_from_slice(&[0u8; 12]);
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());

        let disasm = disassemble_code(&code);
        // Should disassemble the zeros, not collapse them (since they're below threshold)
        assert!(disasm.contains("addi a0, zero, 5"));
        assert!(disasm.contains("addi a1, zero, 10"));
        // Should contain unknown instructions for the zero bytes (not collapsed)
        assert!(disasm.contains("unknown"));
        // Should NOT contain zero run comment (below 16-byte threshold)
        assert!(!disasm.contains("zero bytes"));
    }

    #[test]
    fn test_disassemble_zero_run_with_label() {
        use alloc::{collections::BTreeMap, string::ToString, vec::Vec};

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // Add 20 bytes of zeros starting at offset 4
        code.extend_from_slice(&[0u8; 20]);
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());

        let labels = BTreeMap::from([(0x0004, "padding".to_string())]);
        let disasm = disassemble_code_with_labels(&code, Some(&labels), true);

        assert!(disasm.contains("padding:"));
        assert!(disasm.contains("zero bytes"));
        assert!(disasm.contains("addi a1, zero, 10"));
    }

    #[test]
    fn test_disassemble_multiple_zero_runs() {
        use alloc::vec::Vec;

        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // First zero run: 20 bytes
        code.extend_from_slice(&[0u8; 20]);
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());
        // Second zero run: 16 bytes
        code.extend_from_slice(&[0u8; 16]);
        code.extend_from_slice(&addi(Gpr::A2, Gpr::Zero, 15).to_le_bytes());

        let disasm = disassemble_code(&code);
        assert!(disasm.contains("addi a0, zero, 5"));
        // Should have two zero run comments
        let zero_count = disasm.matches("zero bytes").count();
        assert!(zero_count >= 2);
        assert!(disasm.contains("addi a1, zero, 10"));
        assert!(disasm.contains("addi a2, zero, 15"));
    }

    #[test]
    fn test_disassemble_zero_run_round_trip() {
        use alloc::vec::Vec;

        use super::super::asm_parser::assemble_code;

        // Create code with actual zero bytes (not .org gaps, which don't create zeros)
        let mut code = Vec::new();
        code.extend_from_slice(&addi(Gpr::A0, Gpr::Zero, 5).to_le_bytes());
        // Add 20 bytes of zeros (above 16-byte threshold)
        code.extend_from_slice(&[0u8; 20]);
        code.extend_from_slice(&addi(Gpr::A1, Gpr::Zero, 10).to_le_bytes());

        // Disassemble
        let disasm = disassemble_code(&code);

        // Should show zero run comment and .org directive
        assert!(disasm.contains("zero bytes"));
        assert!(disasm.contains(".org"));

        // The disassembly should be reassemblable, but we need to filter out comments
        // since the assembler only supports # comments, not ; comments
        let mut reassemblable = String::new();
        for line in disasm.lines() {
            let trimmed = line.trim();
            // Skip comment lines (starting with ;)
            if trimmed.starts_with(';') {
                continue;
            }
            // Skip address prefixes (0x0000:)
            if let Some(colon_pos) = trimmed.find(':') {
                if trimmed[..colon_pos].starts_with("0x") {
                    // Extract instruction part after address
                    let inst_part = &trimmed[colon_pos + 1..].trim_start();
                    if !inst_part.is_empty() {
                        reassemblable.push_str(inst_part);
                        reassemblable.push('\n');
                    }
                    continue;
                }
            }
            // Keep everything else (.org, labels, instructions without addresses)
            if !trimmed.is_empty() {
                reassemblable.push_str(line);
                reassemblable.push('\n');
            }
        }

        // Reassemble the filtered disassembly
        let code2 = assemble_code(&reassemblable, None).unwrap();

        // Note: The reassembled code will be shorter because .org doesn't fill gaps with zeros
        // But the instructions should be at the correct addresses via .org directives
        // Verify the first instruction matches
        assert_eq!(&code[0..4], &code2[0..4], "First instruction should match");
        // The second instruction should also match (it will be at offset 0x0018 due to .org)
        // But since .org doesn't create zeros, code2 will only have 8 bytes total
        assert!(
            code2.len() >= 8,
            "Reassembled code should have at least 2 instructions"
        );
        assert_eq!(
            &code[code.len() - 4..],
            &code2[code2.len() - 4..],
            "Last instruction should match"
        );
    }
}
