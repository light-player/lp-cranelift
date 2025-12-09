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

    // 1. Set up stack pointer: lui sp, 0x80010; addi sp, sp, 0
    // lui sp, 0x80010 (load upper 20 bits of 0x80010000 into sp)
    code.extend_from_slice(&encode_lui(2, 0x80010)); // sp = x2
    // addi sp, sp, 0 (add lower 12 bits, which are 0)
    code.extend_from_slice(&encode_addi(2, 2, 0));

    // 2. Call test function: jal ra, test_func_addr
    // Use jalr with absolute address (auipc + addi + jalr)
    // auipc t0, (test_func_addr >> 12)
    code.extend_from_slice(&encode_auipc(5, test_func_addr >> 12)); // t0 = x5
    // addi t0, t0, (test_func_addr & 0xFFF)
    code.extend_from_slice(&encode_addi(5, 5, (test_func_addr & 0xFFF) as i32));
    // jalr ra, t0, 0
    code.extend_from_slice(&encode_jalr(1, 5, 0)); // ra = x1

    // 3. Store result at RESULT_ADDR
    match return_type {
        ReturnType::Int | ReturnType::Bool => {
            // Result is in a0 (x10), store it
            // sw a0, RESULT_ADDR(t0) - but we need a base register
            // Use t0 as base: lui t0, (RESULT_ADDR >> 12); addi t0, t0, (RESULT_ADDR & 0xFFF)
            // encode_lui already shifts by 12, so pass full address
            code.extend_from_slice(&encode_lui(5, RESULT_ADDR));
            code.extend_from_slice(&encode_addi(5, 5, (RESULT_ADDR & 0xFFF) as i32));
            code.extend_from_slice(&encode_sw(5, 10, 0)); // sw a0, 0(t0)
        }
        ReturnType::Float => {
            match fixed_point_format {
                Some(FixedPointFormat::Fixed16x16) => {
                    // Result is in a0 (i32 fixed-point), store it
                    // encode_lui already shifts by 12, so pass full address
                    code.extend_from_slice(&encode_lui(5, RESULT_ADDR));
                    code.extend_from_slice(&encode_addi(5, 5, (RESULT_ADDR & 0xFFF) as i32));
                    code.extend_from_slice(&encode_sw(5, 10, 0)); // sw a0, 0(t0)
                }
                Some(FixedPointFormat::Fixed32x32) => {
                    // Result is in a0 (low) and a1 (high), store both
                    // encode_lui already shifts by 12, so pass full address
                    code.extend_from_slice(&encode_lui(5, RESULT_ADDR));
                    code.extend_from_slice(&encode_addi(5, 5, (RESULT_ADDR & 0xFFF) as i32));
                    code.extend_from_slice(&encode_sw(5, 10, 0)); // sw a0, 0(t0) - low
                    code.extend_from_slice(&encode_sw(5, 11, 4)); // sw a1, 4(t0) - high
                }
                None => {
                    // Result is in fa0 (f32), store it
                    // fsw fa0, RESULT_ADDR(t0)
                    // encode_lui already shifts by 12, so pass full address
                    code.extend_from_slice(&encode_lui(5, RESULT_ADDR));
                    code.extend_from_slice(&encode_addi(5, 5, (RESULT_ADDR & 0xFFF) as i32));
                    code.extend_from_slice(&encode_fsw(5, 10, 0)); // fsw fa0, 0(t0)
                }
            }
        }
        ReturnType::I64 => {
            // Result is in a0 (low) and a1 (high), store both
            // encode_lui already shifts by 12, so pass full address
            code.extend_from_slice(&encode_lui(5, RESULT_ADDR));
            code.extend_from_slice(&encode_addi(5, 5, (RESULT_ADDR & 0xFFF) as i32));
            code.extend_from_slice(&encode_sw(5, 10, 0)); // sw a0, 0(t0) - low
            code.extend_from_slice(&encode_sw(5, 11, 4)); // sw a1, 4(t0) - high
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
    code.extend_from_slice(&[0x73, 0x10, 0x00, 0x00]);

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
