//! Bootstrap code generation for emulator execution
//!
//! Generates riscv32 code that:
//! 1. Sets up stack pointer
//! 2. Calls the test function
//! 3. Stores result in memory
//! 4. Executes EBREAK

use super::backend::ReturnType;
use anyhow::Result;
use lp_glsl::FixedPointFormat;

/// Memory layout constants (standard RISC-V)
pub const RESULT_ADDR: u32 = 0x80001000; // Result storage in RAM
pub const STACK_BASE: u32 = 0x80010000; // Top of stack (grows downward)

/// Generate bootstrap code for riscv32
pub fn generate_bootstrap(
    test_func_addr: u32,
    return_type: ReturnType,
    fixed_point_format: Option<FixedPointFormat>,
) -> Result<Vec<u8>> {
    let mut code = Vec::new();

    // 1. Set up stack pointer: lui sp, (STACK_BASE >> 12); addi sp, sp, 0
    // lui sp, (STACK_BASE >> 12) - load upper 20 bits of STACK_BASE into sp
    // encode_lui expects the full 32-bit value and extracts upper 20 bits
    code.extend_from_slice(&encode_lui(2, STACK_BASE)); // sp = x2
    // addi sp, sp, 0 (add lower 12 bits, which are 0)
    code.extend_from_slice(&encode_addi(2, 2, 0));

    // 2. Call test function: jal ra, test_func_addr
    // Use jalr with absolute address (auipc + addi + jalr)
    // PC-relative addressing: auipc computes PC + (imm << 12)
    // PC at auipc instruction = 8 (after 2 stack setup instructions)
    let pc_at_auipc = code.len() as u32;
    let offset_to_test_func = test_func_addr.wrapping_sub(pc_at_auipc);
    
    // Debug logging (enabled via LP_GLSL_DEBUG env var)
    if std::env::var("LP_GLSL_DEBUG").is_ok() {
        eprintln!("[bootstrap] test_func_addr=0x{:08x}, pc_at_auipc=0x{:08x}, offset=0x{:08x}", 
                  test_func_addr, pc_at_auipc, offset_to_test_func);
    }
    
    // auipc t0, (offset >> 12) - computes t0 = PC + (offset >> 12) << 12
    code.extend_from_slice(&encode_auipc(5, offset_to_test_func >> 12)); // t0 = x5
    // addi t0, t0, (offset & 0xFFF) - adds lower 12 bits
    code.extend_from_slice(&encode_addi(5, 5, (offset_to_test_func & 0xFFF) as i32));
    // jalr ra, t0, 0 - jumps to t0, sets ra = PC + 4
    code.extend_from_slice(&encode_jalr(1, 5, 0)); // ra = x1

    // 3. Store result at RESULT_ADDR
    // Use t1 (x6) instead of t0 to avoid conflicts with function call setup
    // lui t1, (RESULT_ADDR >> 12); addi t1, t1, (RESULT_ADDR & 0xFFF)
    // encode_lui expects the full 32-bit value and extracts upper 20 bits
    let result_base_reg = 6; // t1 = x6
    code.extend_from_slice(&encode_lui(result_base_reg, RESULT_ADDR));
    code.extend_from_slice(&encode_addi(result_base_reg, result_base_reg, (RESULT_ADDR & 0xFFF) as i32));
    
    match return_type {
        ReturnType::Int | ReturnType::Bool => {
            // Result is in a0 (x10), store it
            code.extend_from_slice(&encode_sw(result_base_reg, 10, 0)); // sw a0, 0(t1)
        }
        ReturnType::Float => {
            match fixed_point_format {
                Some(FixedPointFormat::Fixed16x16) => {
                    // Result is in a0 (i32 fixed-point), store it
                    code.extend_from_slice(&encode_sw(result_base_reg, 10, 0)); // sw a0, 0(t1)
                }
                Some(FixedPointFormat::Fixed32x32) => {
                    // Result is in a0 (low) and a1 (high), store both
                    code.extend_from_slice(&encode_sw(result_base_reg, 10, 0)); // sw a0, 0(t1) - low
                    code.extend_from_slice(&encode_sw(result_base_reg, 11, 4)); // sw a1, 4(t1) - high
                }
                None => {
                    // Result is in fa0 (f32), store it
                    code.extend_from_slice(&encode_fsw(result_base_reg, 10, 0)); // fsw fa0, 0(t1)
                }
            }
        }
        ReturnType::I64 => {
            // Result is in a0 (low) and a1 (high), store both
            code.extend_from_slice(&encode_sw(result_base_reg, 10, 0)); // sw a0, 0(t1) - low
            code.extend_from_slice(&encode_sw(result_base_reg, 11, 4)); // sw a1, 4(t1) - high
        }
        ReturnType::Vec2 | ReturnType::Mat2 => {
            // Result is in memory (struct return), copy to RESULT_ADDR
            // For struct return, result pointer is in a0, copy 8 bytes
            // For now, assume result is already at RESULT_ADDR (simplified)
            // TODO: Handle struct return properly
        }
        ReturnType::Vec3 | ReturnType::Mat3 => {
            // Similar to Vec2, but 12/36 bytes
        }
        ReturnType::Vec4 | ReturnType::Mat4 => {
            // Similar to Vec2, but 16/64 bytes
        }
    }

    // 4. EBREAK: ebreak (0x00100073)
    // Encoding: 0x00100073 in little-endian bytes
    code.extend_from_slice(&[0x73, 0x00, 0x10, 0x00]);

    Ok(code)
}

// RISC-V instruction encoding helpers
fn encode_lui(rd: u32, imm: u32) -> [u8; 4] {
    let imm20 = (imm >> 12) & 0xFFFFF;
    let inst = 0x37 | (rd << 7) | (imm20 << 12);
    inst.to_le_bytes()
}

fn encode_addi(rd: u32, rs1: u32, imm: i32) -> [u8; 4] {
    let imm12 = (imm as u32) & 0xFFF;
    let inst = 0x13 | (rd << 7) | (0b000 << 12) | (rs1 << 15) | (imm12 << 20);
    inst.to_le_bytes()
}

fn encode_auipc(rd: u32, imm: u32) -> [u8; 4] {
    let imm20 = (imm >> 12) & 0xFFFFF;
    let inst = 0x17 | (rd << 7) | (imm20 << 12);
    inst.to_le_bytes()
}

fn encode_jalr(rd: u32, rs1: u32, imm: i32) -> [u8; 4] {
    let imm12 = (imm as u32) & 0xFFF;
    let inst = 0x67 | (rd << 7) | (0b000 << 12) | (rs1 << 15) | (imm12 << 20);
    inst.to_le_bytes()
}

fn encode_sw(rs1: u32, rs2: u32, imm: i32) -> [u8; 4] {
    let imm12 = (imm as u32) & 0xFFF;
    let imm11_5 = (imm12 >> 5) & 0x7F;
    let imm4_0 = imm12 & 0x1F;
    let inst = 0x23 | (imm4_0 << 7) | (0b010 << 12) | (rs1 << 15) | (rs2 << 20) | (imm11_5 << 25);
    inst.to_le_bytes()
}

fn encode_fsw(rs1: u32, rs2: u32, imm: i32) -> [u8; 4] {
    // fsw uses same encoding as sw but with different funct3
    let imm12 = (imm as u32) & 0xFFF;
    let imm11_5 = (imm12 >> 5) & 0x7F;
    let imm4_0 = imm12 & 0x1F;
    let inst = 0x27 | (imm4_0 << 7) | (0b010 << 12) | (rs1 << 15) | (rs2 << 20) | (imm11_5 << 25);
    inst.to_le_bytes()
}
