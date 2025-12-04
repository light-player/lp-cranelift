# Add RVC (Compressed Instruction) Support to lp-riscv-tools

## Overview

Add support for RISC-V Compressed (RVC) instructions to the lp-riscv-tools emulator. This will enable running programs compiled for `riscv32imac` targets, which use 16-bit compressed instructions alongside standard 32-bit instructions.

## Background

The RISC-V "C" extension (RVC) provides 16-bit encodings of common 32-bit instructions to reduce code size. Key characteristics:

- **16-bit instructions** alongside standard 32-bit instructions
- **Identified by bits [1:0]**: If `!= 0b11`, it's a compressed instruction
- **~40 instruction variants** across 3 formats (CR, CI, CSS, CIW, CL, CS, CB, CJ)
- **Widely used**: Standard library and most RISC-V code includes RVC

## Current Blocker

The hello world test fails at PC=604 with:

```
InvalidInstruction { instruction: 289505410, reason: "Unknown opcode: 0x02" }
```

Opcode 0x02 indicates a compressed instruction that expands to a standard instruction.

## Implementation Steps

### 1. Add RVC Instruction Types

**[crates/lp-riscv-tools/src/inst.rs](crates/lp-riscv-tools/src/inst.rs)**

Add enum variants for compressed instructions:

```rust
pub enum Inst {
    // ... existing instructions ...

    // Compressed instructions (expand to standard forms)
    CAddi { rd: Gpr, imm: i32 },        // c.addi -> addi
    CLi { rd: Gpr, imm: i32 },          // c.li -> addi rd, x0, imm
    CLui { rd: Gpr, imm: i32 },         // c.lui -> lui
    CMv { rd: Gpr, rs: Gpr },           // c.mv -> add rd, x0, rs
    CAdd { rd: Gpr, rs: Gpr },          // c.add -> add
    CSub { rd: Gpr, rs: Gpr },          // c.sub -> sub
    CAnd { rd: Gpr, rs: Gpr },          // c.and -> and
    COr { rd: Gpr, rs: Gpr },           // c.or -> or
    CXor { rd: Gpr, rs: Gpr },          // c.xor -> xor
    CLw { rd: Gpr, rs: Gpr, offset: i32 }, // c.lw -> lw
    CSw { rs1: Gpr, rs2: Gpr, offset: i32 }, // c.sw -> sw
    CJ { offset: i32 },                 // c.j -> jal x0, offset
    CJr { rs: Gpr },                    // c.jr -> jalr x0, rs, 0
    CJalr { rs: Gpr },                  // c.jalr -> jalr x1, rs, 0
    CBeqz { rs: Gpr, offset: i32 },     // c.beqz -> beq rs, x0, offset
    CBnez { rs: Gpr, offset: i32 },     // c.bnez -> bne rs, x0, offset
    // Add more as needed
}
```

### 2. Create RVC Decoder

**[crates/lp-riscv-tools/src/decode_rvc.rs](crates/lp-riscv-tools/src/decode_rvc.rs)** (new file)

```rust
//! Decoder for RISC-V Compressed (RVC) instructions.

use crate::inst::Inst;
use crate::regs::Gpr;
use alloc::string::String;

/// Decode a 16-bit compressed instruction.
pub fn decode_compressed(inst: u16) -> Result<Inst, String> {
    let opcode = inst & 0x3; // bits [1:0]
    let funct3 = (inst >> 13) & 0x7; // bits [15:13]

    match opcode {
        0b00 => decode_c0(inst, funct3),
        0b01 => decode_c1(inst, funct3),
        0b10 => decode_c2(inst, funct3),
        _ => unreachable!(), // 0b11 is not compressed
    }
}

fn decode_c0(inst: u16, funct3: u16) -> Result<Inst, String> {
    match funct3 {
        0b010 => decode_c_lw(inst),      // C.LW
        0b110 => decode_c_sw(inst),      // C.SW
        // ... other C0 quadrant instructions
        _ => Err(format!("Unknown C0 instruction: funct3={:03b}", funct3)),
    }
}

fn decode_c1(inst: u16, funct3: u16) -> Result<Inst, String> {
    match funct3 {
        0b000 => decode_c_addi(inst),    // C.ADDI / C.NOP
        0b001 => decode_c_jal(inst),     // C.JAL
        0b010 => decode_c_li(inst),      // C.LI
        0b101 => decode_c_j(inst),       // C.J
        0b110 => decode_c_beqz(inst),    // C.BEQZ
        0b111 => decode_c_bnez(inst),    // C.BNEZ
        // ... other C1 quadrant instructions
        _ => Err(format!("Unknown C1 instruction: funct3={:03b}", funct3)),
    }
}

fn decode_c2(inst: u16, funct3: u16) -> Result<Inst, String> {
    match funct3 {
        0b100 => {
            // C.JR, C.MV, C.JALR, C.ADD
            let rs1 = ((inst >> 7) & 0x1f) as u8;
            let rs2 = ((inst >> 2) & 0x1f) as u8;

            if rs2 == 0 {
                if rs1 == 0 {
                    // C.EBREAK - but this should be 0x9002
                    Err("Unexpected C.EBREAK".into())
                } else {
                    // C.JR / C.JALR
                    let is_jalr = (inst & (1 << 12)) != 0;
                    if is_jalr {
                        Ok(Inst::CJalr { rs: Gpr::new(rs1) })
                    } else {
                        Ok(Inst::CJr { rs: Gpr::new(rs1) })
                    }
                }
            } else {
                // C.MV / C.ADD
                let rd = rs1;
                let is_add = (inst & (1 << 12)) != 0;
                if is_add {
                    Ok(Inst::CAdd {
                        rd: Gpr::new(rd),
                        rs: Gpr::new(rs2)
                    })
                } else {
                    Ok(Inst::CMv {
                        rd: Gpr::new(rd),
                        rs: Gpr::new(rs2)
                    })
                }
            }
        }
        // ... other C2 quadrant instructions
        _ => Err(format!("Unknown C2 instruction: funct3={:03b}", funct3)),
    }
}

// Helper functions to decode specific instruction types
fn decode_c_addi(inst: u16) -> Result<Inst, String> {
    let rd = ((inst >> 7) & 0x1f) as u8;
    if rd == 0 {
        return Ok(Inst::Nop); // C.NOP
    }

    // Extract immediate: imm[5] | imm[4:0]
    let imm5 = ((inst >> 12) & 0x1) as i32;
    let imm4_0 = ((inst >> 2) & 0x1f) as i32;
    let imm = (imm5 << 5) | imm4_0;

    // Sign extend from bit 5
    let imm = if (imm & 0x20) != 0 {
        imm | !0x3f
    } else {
        imm
    };

    Ok(Inst::CAddi { rd: Gpr::new(rd), imm })
}

// ... more helper functions for each instruction type
```

### 3. Update Main Decoder

**[crates/lp-riscv-tools/src/decode.rs](crates/lp-riscv-tools/src/decode.rs)**

```rust
pub fn decode_instruction(inst: u32) -> Result<Inst, String> {
    // Check if this is a compressed instruction
    if (inst & 0x3) != 0x3 {
        // It's a 16-bit compressed instruction
        return crate::decode_rvc::decode_compressed(inst as u16);
    }

    // Standard 32-bit instruction decoding
    let opcode = (inst & 0x7f) as u8;
    // ... existing code ...
}
```

### 4. Update Emulator to Handle 16-bit Instructions

**[crates/lp-riscv-tools/src/emu/emulator.rs](crates/lp-riscv-tools/src/emu/emulator.rs)**

Modify `step()` to handle variable-length instructions:

```rust
pub fn step(&mut self) -> Result<StepResult, EmulatorError> {
    // ... existing code ...

    // Fetch instruction (might be 16 or 32 bits)
    let inst_word = self.memory.fetch_instruction(self.pc)?;

    // Check if compressed
    let is_compressed = (inst_word & 0x3) != 0x3;

    // Decode instruction
    let decoded = decode_instruction(inst_word)?;

    // ... existing code ...

    // Update PC (2 bytes for compressed, 4 for standard)
    let pc_increment = if is_compressed { 2 } else { 4 };
    self.pc = exec_result.new_pc.unwrap_or(self.pc.wrapping_add(pc_increment));

    // ... existing code ...
}
```

### 5. Update Executor

**[crates/lp-riscv-tools/src/emu/executor.rs](crates/lp-riscv-tools/src/emu/executor.rs)**

Add execution for compressed instructions. Most expand to existing instruction forms:

```rust
pub fn execute_instruction(
    inst: Inst,
    pc: u32,
    regs: &mut [i32; 32],
    memory: &mut Memory,
) -> Result<ExecutionResult, EmulatorError> {
    match inst {
        // ... existing instructions ...

        Inst::CAddi { rd, imm } => {
            // c.addi expands to: addi rd, rd, imm
            let result = regs[rd.num() as usize].wrapping_add(imm);
            regs[rd.num() as usize] = result;
            Ok(ExecutionResult::normal(/* log */))
        }

        Inst::CMv { rd, rs } => {
            // c.mv expands to: add rd, x0, rs
            regs[rd.num() as usize] = regs[rs.num() as usize];
            Ok(ExecutionResult::normal(/* log */))
        }

        Inst::CAdd { rd, rs } => {
            // c.add expands to: add rd, rd, rs
            let result = regs[rd.num() as usize]
                .wrapping_add(regs[rs.num() as usize]);
            regs[rd.num() as usize] = result;
            Ok(ExecutionResult::normal(/* log */))
        }

        // ... more compressed instructions ...
    }
}
```

### 6. Add Module Declaration

**[crates/lp-riscv-tools/src/lib.rs](crates/lp-riscv-tools/src/lib.rs)**

```rust
pub mod decode;
pub mod decode_rvc;  // Add this
```

### 7. Add Tests

**[crates/lp-riscv-tools/src/decode_rvc.rs](crates/lp-riscv-tools/src/decode_rvc.rs)**

Add unit tests for each compressed instruction:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_addi() {
        // c.addi x10, 5 -> 0x0515
        let inst = decode_compressed(0x0515).unwrap();
        // Should decode to CAddi { rd: x10, imm: 5 }
    }

    #[test]
    fn test_c_add() {
        // c.add x10, x11 -> 0x956a (example)
        // Should decode to CAdd { rd: x10, rs: x11 }
    }

    // ... tests for all instruction types
}
```

## Priority Instruction List

Focus on implementing the most common compressed instructions first:

### High Priority (Required for stdlib)

1. **C.ADDI** - Add immediate
2. **C.LI** - Load immediate
3. **C.LUI** - Load upper immediate
4. **C.MV** - Move (copy register)
5. **C.ADD** - Add registers
6. **C.LW** - Load word
7. **C.SW** - Store word
8. **C.J** - Jump
9. **C.JR** - Jump register
10. **C.JALR** - Jump and link register
11. **C.BEQZ** - Branch if equal to zero
12. **C.BNEZ** - Branch if not equal to zero

### Medium Priority

13. **C.ADDI16SP** - Adjust stack pointer
14. **C.LWSP** - Load word from stack
15. **C.SWSP** - Store word to stack
16. **C.SLLI** - Shift left logical immediate
17. **C.SRLI** - Shift right logical immediate
18. **C.SRAI** - Shift right arithmetic immediate
19. **C.ANDI** - AND immediate
20. **C.SUB** - Subtract
21. **C.XOR** - Exclusive OR
22. **C.OR** - OR
23. **C.AND** - AND

### Low Priority (Less Common)

24. **C.JAL** - Jump and link (RV32 only)
25. **C.ADDI4SPN** - Add immediate to SP, non-destructive
26. **C.EBREAK** - Breakpoint

## Testing Strategy

### 1. Unit Tests

Test each compressed instruction decoder individually with known encodings

### 2. Integration Test

The existing `riscv_nostd_test` should pass once RVC support is complete:

```bash
cargo test --package lp-riscv-tools --features std riscv_nostd -- --ignored --nocapture
```

Expected output:

```
✅ Test Passed!
Successfully ran no_std Cranelift-compiled code on RISC-V emulator
```

### 3. Validation Tests

Create specific tests for:

- Mixed 32-bit and 16-bit instructions
- Aligned and unaligned compressed instructions
- All quadrants (C0, C1, C2)
- Edge cases (nop, illegal instructions)

## Reference Materials

- **RISC-V Spec Volume I Chapter 16**: RVC Extension
- **Opcodes**:
  - Quadrant 0: `0b00`
  - Quadrant 1: `0b01`
  - Quadrant 2: `0b10`
- **Instruction Formats**: CR, CI, CSS, CIW, CL, CS, CA, CB, CJ

## Implementation Order

1. Create `decode_rvc.rs` with basic structure
2. Implement C2 quadrant (C.MV, C.ADD, C.JR, C.JALR) - most common
3. Implement C1 quadrant (C.ADDI, C.LI, C.J, C.BEQZ, C.BNEZ)
4. Implement C0 quadrant (C.LW, C.SW)
5. Update emulator PC increment logic
6. Add executor cases
7. Run tests and iterate

## Success Criteria

- ✅ `riscv_nostd_test` passes
- ✅ Hello world program runs to completion
- ✅ Output contains "Hello from RISC-V!"
- ✅ Program exits cleanly with EBREAK
- ✅ No emulator errors

## Estimated Effort

- Basic implementation (top 12 instructions): ~2-3 hours
- Full implementation (all common instructions): ~4-6 hours
- Testing and debugging: ~2 hours
- **Total**: ~6-8 hours

This will unblock Phase 1 of the embive-program demo and enable running real no_std Rust code in the emulator.

