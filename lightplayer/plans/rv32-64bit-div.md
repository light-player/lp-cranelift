---
name: i64-division-riscv32
overview: Implement full 64-bit division and remainder operations for RISC-V32 using register pairs, with optimized paths for common cases and a full implementation for arbitrary 64-bit divisors.
todos:
  - id: add-minst-variants
    content: Add MInst enum variants (DivI64U, DivI64S, RemI64U, RemI64S) to inst.isle with temp registers for quotient, remainder, counter, and sign
    status: pending
  - id: add-isle-helpers
    content: Add ISLE helper declarations (div_i64_inst, div_i64_signed_inst, rem_i64_inst, rem_i64_signed_inst) that emit MInst variants
    status: pending
  - id: implement-64bit-shift
    content: Implement 64-bit left shift helper for register pairs (needed for binary division algorithm)
    status: pending
  - id: implement-binary-division
    content: Implement binary long division algorithm in emit.rs for DivI64U instruction (follow AtomicRmwSeq/BrTable pattern)
    status: pending
  - id: implement-signed-division
    content: Implement signed division in emit.rs for DivI64S instruction (handle signs, overflow, division by zero)
    status: pending
  - id: implement-remainder
    content: Implement remainder operations in emit.rs (RemI64U, RemI64S) - can reuse division algorithm
    status: pending
  - id: add-lowering-rules
    content: Add lowering rules in lower.isle for udiv/sdiv/urem/srem with i64 type
    status: pending
  - id: add-safe-divisor-helper
    content: Add safe_sdiv_divisor_i64 helper function for checking safe signed division conditions
    status: pending
  - id: implement-operand-tracking
    content: Add operand tracking in riscv32_get_operands for new division instructions (reg_use for inputs, reg_early_def for outputs/temps)
    status: pending
  - id: handle-worst-case-size
    content: Handle worst_case_size() for division instructions (either emit island or exclude from check)
    status: pending
  - id: implement-display
    content: Implement Display/Debug for new DivI64U/DivI64S/RemI64U/RemI64S instructions
    status: pending
  - id: test-division
    content: Test i64 division operations with filetests (urem.clif, div-checks.clif)
    status: completed
---

# Plan: Full 64-bit Division Support for RISC-V32

## Overview

RISC-V32 only provides native 32-bit division instructions (`div`, `divu`, `rem`, `remu`). To support i64 division, we need to implement software division using register pairs, similar to how `sub_i64` handles 64-bit subtraction.

## Architecture Analysis

### RISC-V32 Division Instructions

- `div rd, rs1, rs2`: 32-bit signed division (rd = rs1 /s rs2)
- `divu rd, rs1, rs2`: 32-bit unsigned division (rd = rs1 /u rs2)
- `rem rd, rs1, rs2`: 32-bit signed remainder (rd = rs1 %s rs2)
- `remu rd, rs1, rs2`: 32-bit unsigned remainder (rd = rs1 %u rs2)

### Reference Implementations

**x64 approach** (hardware support):

- Uses `div`/`idiv` instructions that take 128-bit dividend in `rax:rdx` and 64-bit divisor
- Sign-extends dividend into `rdx` for signed division
- Returns quotient in `rax`, remainder in `rdx`

**riscv32 i64 approach** (software implementation needed):

- Mirror riscv64's i128 pattern (which uses register pairs)
- Use binary long division algorithm for full 64-bit support
- Optimize for common case: divisor fits in 32 bits

## Implementation Strategy

**Decision: Implement in Rust** (not ISLE)

Following the pattern used by other complex multi-instruction sequences in Cranelift (e.g., `AtomicRmwSeq` in x64, `BrTable` in riscv32), we'll implement i64 division directly in Rust in `emit.rs`. This approach:

- Avoids ISLE's loop limitations (would require unrolling 64 iterations)
- Provides better control over the algorithm
- Matches existing patterns in the codebase
- Allows for easier optimization and maintenance

### Algorithm: Binary Long Division (Restoring Division)

For arbitrary 64-bit divisors:

1. Initialize: `quotient = 0`, `remainder = dividend`
2. For 64 iterations (bit positions 63 down to 0):

   - Shift `remainder` left by 1 (brings in next bit from dividend implicitly)
   - Shift `quotient` left by 1
   - If `remainder >= divisor`:
     - `remainder = remainder - divisor`
     - Set LSB of `quotient` to 1 (OR quotient with 1)

**Note**: In restoring division, we start with `remainder = dividend`, so shifting remainder left each iteration automatically brings in the next bit from the original dividend. No explicit bit extraction is needed.

### Signed Division

1. Check for special cases (divisor = 0, INT_MIN / -1 overflow) - handled by `safe_sdiv_divisor_i64`
2. Compute sign of result: `sign = (dividend < 0) XOR (divisor < 0)`
3. Convert to unsigned: take absolute values of dividend and divisor
   - For 64-bit: if negative, negate using two's complement (invert and add 1)
4. Perform unsigned division
5. Apply sign to quotient: if sign is negative, negate the quotient
6. For remainder: sign matches dividend (not quotient)

### Remainder

- For unsigned: remainder is the final remainder from division algorithm
- For signed: remainder = dividend - (quotient * divisor), with sign of dividend
  - Can be computed by: if dividend < 0, negate the remainder from unsigned division

## Implementation Steps

### Step 1: Add MInst Enum Variants

File: `cranelift/codegen/src/isa/riscv32/inst.isle` (in MInst enum)

Add new instruction variants to the `MInst` enum. These will need temp registers allocated during lowering:

```isle
(DivI64U
  (dividend ValueRegs)  ; (low, high) register pair - input
  (divisor ValueRegs)   ; (low, high) register pair - input
  (quotient ValueRegs)  ; (low, high) register pair - output
  (rem ValueRegs)       ; (low, high) register pair - temp (for remainder calculation)
  (counter WritableReg)) ; temp register for loop counter

(DivI64S
  (dividend ValueRegs)
  (divisor ValueRegs)
  (quotient ValueRegs)
  (rem ValueRegs)
  (counter WritableReg)
  (sign WritableReg))    ; temp to store sign information

(RemI64U
  (dividend ValueRegs)
  (divisor ValueRegs)
  (remainder ValueRegs) ; (low, high) register pair - output
  (quotient ValueRegs)  ; temp for quotient calculation
  (counter WritableReg))

(RemI64S
  (dividend ValueRegs)
  (divisor ValueRegs)
  (remainder ValueRegs)
  (quotient ValueRegs)
  (counter WritableReg)
  (sign WritableReg))
```

**Note**: `ValueRegs` is used for both inputs and outputs. In emit.rs, we convert to `WritableXReg` when needed using `writable_xreg_new(Writable::from_reg(reg))`.

### Step 2: Add ISLE Helper Declarations

File: `cranelift/codegen/src/isa/riscv32/inst.isle`

Add helper declarations that emit the MInst variants:

```isle
;; 64-bit unsigned division instruction
(decl div_i64_inst (ValueRegs ValueRegs) ValueRegs)
(rule (div_i64_inst dividend divisor)
  (let (
      (quotient ValueRegs (temp_value_regs $I64))
      (rem ValueRegs (temp_value_regs $I64))
      (counter WritableReg (temp_writable_reg))
      (_ InstOutput (emit (MInst.DivI64U dividend divisor quotient rem counter)))
    )
    quotient))

;; 64-bit signed division instruction
(decl div_i64_signed_inst (ValueRegs ValueRegs) ValueRegs)
(rule (div_i64_signed_inst dividend divisor)
  (let (
      (quotient ValueRegs (temp_value_regs $I64))
      (rem ValueRegs (temp_value_regs $I64))
      (counter WritableReg (temp_writable_reg))
      (sign WritableReg (temp_writable_reg))
      (_ InstOutput (emit (MInst.DivI64S dividend divisor quotient rem counter sign)))
    )
    quotient))

;; 64-bit unsigned remainder instruction
(decl rem_i64_inst (ValueRegs ValueRegs) ValueRegs)
(rule (rem_i64_inst dividend divisor)
  (let (
      (remainder ValueRegs (temp_value_regs $I64))
      (quotient ValueRegs (temp_value_regs $I64))
      (counter WritableReg (temp_writable_reg))
      (_ InstOutput (emit (MInst.RemI64U dividend divisor remainder quotient counter)))
    )
    remainder))

;; 64-bit signed remainder instruction
(decl rem_i64_signed_inst (ValueRegs ValueRegs) ValueRegs)
(rule (rem_i64_signed_inst dividend divisor)
  (let (
      (remainder ValueRegs (temp_value_regs $I64))
      (quotient ValueRegs (temp_value_regs $I64))
      (counter WritableReg (temp_writable_reg))
      (sign WritableReg (temp_writable_reg))
      (_ InstOutput (emit (MInst.RemI64S dividend divisor remainder quotient counter sign)))
    )
    remainder))
```

### Step 3: Implement Required Helper Functions

Before implementing division, we need helper functions for 64-bit operations:

**64-bit Left Shift** (register pair):

- Shift left by 1: `(high, low) << 1 = ((high << 1) | (low >> 31), low << 1)`
- Implementation (requires a temp register - use `writable_spilltmp_reg()`):
  ```rust
  let shift_temp = writable_spilltmp_reg();
  
  // Save high bit of low register (MSB)
  Inst::AluRRImm12 {
      alu_op: AluOPRRI::Srli,
      rd: shift_temp,
      rs: rem_lo.to_reg(),
      imm12: Imm12::from_i16(31),
  }
  .emit_uncompressed(sink, emit_info, state, start_off);
  
  // Shift low left by 1
  Inst::AluRRImm12 {
      alu_op: AluOPRRI::Slli,
      rd: rem_lo,
      rs: rem_lo.to_reg(),
      imm12: Imm12::from_i16(1),
  }
  .emit_uncompressed(sink, emit_info, state, start_off);
  
  // Shift high left by 1
  Inst::AluRRImm12 {
      alu_op: AluOPRRI::Slli,
      rd: rem_hi,
      rs: rem_hi.to_reg(),
      imm12: Imm12::from_i16(1),
  }
  .emit_uncompressed(sink, emit_info, state, start_off);
  
  // OR the MSB from low into high
  Inst::AluRRR {
      alu_op: AluOPRRR::Or,
      rd: rem_hi,
      rs1: rem_hi.to_reg(),
      rs2: shift_temp.to_reg(),
  }
  .emit_uncompressed(sink, emit_info, state, start_off);
  ```


**64-bit Comparison** (must be implemented directly in Rust):

- Cannot use ISLE helpers (`lower_icmp_i64`) from Rust emission code
- Must emit comparison instructions directly:
  - Compare high parts first: `cmp_hi = (rem_hi < divisor_hi)` or `(rem_hi == divisor_hi)`
  - If high parts equal, compare low parts: `cmp_lo = (rem_lo < divisor_lo)`
  - Combine: `(rem_hi < divisor_hi) || ((rem_hi == divisor_hi) && (rem_lo < divisor_lo))`
  - For `>=` comparison: negate the result
  - Use `Inst::CondBr` with the comparison result

**64-bit Subtraction** (already exists):

- Use `sub_i64` pattern from `inst.isle`
- Or implement inline using the same pattern (subtract low, compute borrow, subtract high with borrow)

**64-bit Negation** (for signed division):

- Two's complement: invert all bits, then add 1
- For register pair: invert both parts, add 1 to low part, propagate carry to high part

### Step 4: Implement Binary Long Division in Rust

File: `cranelift/codegen/src/isa/riscv32/inst/emit.rs`

Add to `emit_uncompressed` function, following the pattern from `AtomicRmwSeq` and `BrTable`:

```rust
&Inst::DivI64U {
    dividend,
    divisor,
    quotient,
    rem,
    counter,
} => {
    // Extract register pairs from ValueRegs
    // ValueRegs stores registers in parts[0] (low) and parts[1] (high)
    let dividend_regs = dividend.regs();
    let dividend_lo = XReg::new(dividend_regs[0]).unwrap();
    let dividend_hi = XReg::new(dividend_regs[1]).unwrap();
    
    let divisor_regs = divisor.regs();
    let divisor_lo = XReg::new(divisor_regs[0]).unwrap();
    let divisor_hi = XReg::new(divisor_regs[1]).unwrap();
    
    let quotient_regs = quotient.regs();
    let quotient_lo = writable_xreg_new(Writable::from_reg(quotient_regs[0]));
    let quotient_hi = writable_xreg_new(Writable::from_reg(quotient_regs[1]));
    
    let rem_regs = rem.regs();
    let rem_lo = writable_xreg_new(Writable::from_reg(rem_regs[0]));
    let rem_hi = writable_xreg_new(Writable::from_reg(rem_regs[1]));
    
    // Check for division by zero: divisor == 0
    // Check if (divisor_lo | divisor_hi) == 0
    let divisor_zero_check = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::Or,
        rd: divisor_zero_check,
        rs1: divisor_lo,
        rs2: divisor_hi,
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Emit trap if divisor is zero
    Inst::TrapIf {
        rs1: divisor_zero_check.to_reg(),
        rs2: zero_reg(),
        cc: IntCC::Equal,
        trap_code: TrapCode::INTEGER_DIVISION_BY_ZERO,
    }
    .emit(sink, emit_info, state);
    
    // Initialize: remainder = dividend, quotient = 0
    Inst::AluRRR {
        alu_op: AluOPRRR::Add,  // mov via add with zero
        rd: rem_lo,
        rs1: dividend_lo,
        rs2: zero_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    Inst::AluRRR {
        alu_op: AluOPRRR::Add,
        rd: rem_hi,
        rs1: dividend_hi,
        rs2: zero_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Initialize quotient to 0
    Inst::AluRRR {
        alu_op: AluOPRRR::Add,
        rd: quotient_lo,
        rs1: zero_reg(),
        rs2: zero_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    Inst::AluRRR {
        alu_op: AluOPRRR::Add,
        rd: quotient_hi,
        rs1: zero_reg(),
        rs2: zero_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Initialize counter to 64
    Inst::load_constant_u32(*counter, 64)
        .iter()
        .for_each(|i| i.emit_uncompressed(sink, emit_info, state, start_off));
    
    // Allocate temp registers for the loop
    // Note: writable_spilltmp_reg() and writable_spilltmp_reg2() provide two distinct temp registers
    // These can be reused throughout the loop since values are consumed immediately
    let shift_temp = writable_spilltmp_reg();  // For 64-bit shift operations
    
    // Binary long division loop
    let loop_label = sink.get_label();
    let done_label = sink.get_label();
    
    sink.bind_label(loop_label, &mut state.ctrl_plane);
    
    // Shift remainder left by 1 (64-bit shift)
    // Save high bit of low register (MSB)
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Srli,
        rd: shift_temp,
        rs: rem_lo.to_reg(),
        imm12: Imm12::from_i16(31),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Shift low left by 1
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Slli,
        rd: rem_lo,
        rs: rem_lo.to_reg(),
        imm12: Imm12::from_i16(1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Shift high left by 1
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Slli,
        rd: rem_hi,
        rs: rem_hi.to_reg(),
        imm12: Imm12::from_i16(1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // OR the MSB from low into high
    Inst::AluRRR {
        alu_op: AluOPRRR::Or,
        rd: rem_hi,
        rs1: rem_hi.to_reg(),
        rs2: shift_temp.to_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Shift quotient left by 1 (64-bit shift) - same pattern
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Srli,
        rd: shift_temp,
        rs: quotient_lo.to_reg(),
        imm12: Imm12::from_i16(31),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Slli,
        rd: quotient_lo,
        rs: quotient_lo.to_reg(),
        imm12: Imm12::from_i16(1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Slli,
        rd: quotient_hi,
        rs: quotient_hi.to_reg(),
        imm12: Imm12::from_i16(1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    Inst::AluRRR {
        alu_op: AluOPRRR::Or,
        rd: quotient_hi,
        rs1: quotient_hi.to_reg(),
        rs2: shift_temp.to_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Compare remainder >= divisor (64-bit unsigned comparison)
    // Note: We reuse temp registers sequentially - each value is consumed before the next is written
    // Compare high parts first
    let cmp_hi_lt = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::Sltu,
        rd: cmp_hi_lt,
        rs1: rem_hi.to_reg(),
        rs2: divisor_hi,
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    let cmp_hi_eq = writable_spilltmp_reg2();
    Inst::AluRRR {
        alu_op: AluOPRRR::Xor,
        rd: cmp_hi_eq,
        rs1: rem_hi.to_reg(),
        rs2: divisor_hi,
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    // seqz is implemented as sltiu rd, rs, 1
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::SltiU,
        rd: cmp_hi_eq,
        rs: cmp_hi_eq.to_reg(),
        imm12: Imm12::from_i16(1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Compare low parts (only if high parts are equal)
    let cmp_lo_lt = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::Sltu,
        rd: cmp_lo_lt,
        rs1: rem_lo.to_reg(),
        rs2: divisor_lo,
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Combine: (rem_hi < divisor_hi) || ((rem_hi == divisor_hi) && (rem_lo < divisor_lo))
    let cmp_lo_and = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::And,
        rd: cmp_lo_and,
        rs1: cmp_hi_eq.to_reg(),
        rs2: cmp_lo_lt.to_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    let rem_lt_divisor = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::Or,
        rd: rem_lt_divisor,
        rs1: cmp_hi_lt.to_reg(),
        rs2: cmp_lo_and.to_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Branch if remainder < divisor (skip subtraction)
    let skip_subtract_label = sink.get_label();
    Inst::CondBr {
        taken: CondBrTarget::Label(skip_subtract_label),
        not_taken: CondBrTarget::Fallthrough,
        kind: IntegerCompare {
            kind: IntCC::NotEqual,
            rs1: rem_lt_divisor.to_reg(),
            rs2: zero_reg(),
        },
    }
    .emit(sink, emit_info, state);
    
    // remainder >= divisor: subtract divisor from remainder
    // Subtract low parts
    let rem_lo_new = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::Sub,
        rd: rem_lo_new,
        rs1: rem_lo.to_reg(),
        rs2: divisor_lo,
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Compute borrow: rem_lo < divisor_lo (already computed in cmp_lo_lt)
    // Subtract high parts with borrow
    let rem_hi_tmp = writable_spilltmp_reg();
    Inst::AluRRR {
        alu_op: AluOPRRR::Sub,
        rd: rem_hi_tmp,
        rs1: rem_hi.to_reg(),
        rs2: divisor_hi,
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    Inst::AluRRR {
        alu_op: AluOPRRR::Sub,
        rd: rem_hi,
        rs1: rem_hi_tmp.to_reg(),
        rs2: cmp_lo_lt.to_reg(), // borrow
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Update remainder registers
    Inst::AluRRR {
        alu_op: AluOPRRR::Add,
        rd: rem_lo,
        rs1: rem_lo_new.to_reg(),
        rs2: zero_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Set LSB of quotient to 1
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Ori,
        rd: quotient_lo,
        rs: quotient_lo.to_reg(),
        imm12: Imm12::from_i16(1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    sink.bind_label(skip_subtract_label, &mut state.ctrl_plane);
    
    // Decrement counter
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Addi,
        rd: *counter,
        rs: counter.to_reg(),
        imm12: Imm12::from_i16(-1),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Branch if counter != 0
    Inst::CondBr {
        taken: CondBrTarget::Label(loop_label),
        not_taken: CondBrTarget::Label(done_label),
        kind: IntegerCompare {
            kind: IntCC::NotEqual,
            rs1: counter.to_reg(),
            rs2: zero_reg(),
        },
    }
    .emit(sink, emit_info, state);
    
    sink.bind_label(done_label, &mut state.ctrl_plane);
}
```

**Key implementation details**:

- Follow the pattern from `AtomicRmwSeq` (x64) and `BrTable` (riscv32)
- Use labels for loops: `sink.get_label()` and `sink.bind_label()`
- Emit instructions sequentially using `.emit()` or `.emit_uncompressed()`
- Extract registers from ValueRegs: `value_regs.regs()[0]` for low, `[1]` for high
- Convert to XReg: `XReg::new(reg).unwrap()` for read-only registers
- Convert to WritableXReg: For writable ValueRegs, use `writable_xreg_new(Writable::from_reg(value_regs.regs()[0]))`
- Note: ValueRegs in MInst variants will be `ValueRegs<Reg>`, so use `.regs()` to get `&[Reg]`
- **64-bit subtraction**: Implement inline using the borrow pattern (subtract low, compute borrow with `sltu`, subtract high with borrow)
- **64-bit left shift**: Implement inline using temp register (see Step 3)
- **64-bit comparison**: Implement inline in Rust (cannot use ISLE helpers from emission code)
- **Setting quotient bits**: Shift quotient left, then OR in 1 for the LSB when remainder >= divisor
- **Temp registers**: Use `writable_spilltmp_reg()` and `writable_spilltmp_reg2()` for temporary values during emission
- Handle division by zero (trap before loop using `Inst::TrapIf`)
- Handle signed overflow (INT_MIN / -1) in signed version (checked in `safe_sdiv_divisor_i64`)
- **Worst-case size**: Division will likely exceed 84 bytes (current worst_case_size). Exclude from worst-case size checks by adding to `emits_own_island` match in `emit.rs`

### Step 5: Implement Signed Division

File: `cranelift/codegen/src/isa/riscv32/inst/emit.rs`

Add signed division implementation for `DivI64S`:

```rust
&Inst::DivI64S {
    dividend,
    divisor,
    quotient,
    rem,
    counter,
    sign,
} => {
    // Similar setup to DivI64U...
    // Extract registers...
    
    // Check for division by zero and overflow (handled by safe_sdiv_divisor_i64)
    // But we still need to check here as a safety measure
    
    // Compute sign: (dividend < 0) XOR (divisor < 0)
    // Check if dividend is negative (high bit is set)
    let dividend_neg = writable_spilltmp_reg();
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Srli,
        rd: dividend_neg,
        rs: dividend_hi,
        imm12: Imm12::from_i16(31),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Check if divisor is negative
    let divisor_neg = writable_spilltmp_reg2();
    Inst::AluRRImm12 {
        alu_op: AluOPRRI::Srli,
        rd: divisor_neg,
        rs: divisor_hi,
        imm12: Imm12::from_i16(31),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // XOR to get result sign
    Inst::AluRRR {
        alu_op: AluOPRRR::Xor,
        rd: *sign,
        rs1: dividend_neg.to_reg(),
        rs2: divisor_neg.to_reg(),
    }
    .emit_uncompressed(sink, emit_info, state, start_off);
    
    // Convert dividend to absolute value if negative
    // ... (negate if negative) ...
    
    // Convert divisor to absolute value if negative
    // ... (negate if negative) ...
    
    // Perform unsigned division (same as DivI64U)
    // ... (call unsigned division algorithm) ...
    
    // Apply sign to quotient: if sign is negative, negate quotient
    // ... (negate if sign is set) ...
}
```

**Note**: The full signed division implementation should:
1. Compute sign of result
2. Convert both operands to absolute values (negate if negative)
3. Call unsigned division algorithm
4. Negate quotient if result sign is negative
5. For remainder: if dividend was negative, negate remainder

### Step 6: Add Lowering Rules

File: `cranelift/codegen/src/isa/riscv32/lower.isle`

Add lowering rules for i64 division operations:

```isle
;;;; Rules for `udiv` i64 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(rule 2 (lower (has_type $I64 (udiv x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      ;; Check if divisor high bits are zero (optimization)
      (y_hi XReg (value_regs_get y_regs 1))
      (divisor_is_32bit XReg (rv_seqz y_hi))
      ;; For now, always use full division
      ;; TODO: Add optimized path for 32-bit divisor
      (quotient ValueRegs (div_i64_inst x_regs y_regs))
    )
    quotient))

;;;; Rules for `sdiv` i64 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(rule 4 (lower (has_type $I64 (sdiv x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      ;; Check for division by zero and overflow
      (y_safe ValueRegs (safe_sdiv_divisor_i64 x_regs y_regs))
      (quotient ValueRegs (div_i64_signed_inst x_regs y_safe))
    )
    quotient))

;;;; Rules for `urem` i64 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(rule 4 (lower (has_type $I64 (urem x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      ;; Check if entire divisor is zero: (y_lo | y_hi) == 0
      (y_lo XReg (value_regs_get y_regs 0))
      (y_hi XReg (value_regs_get y_regs 1))
      (divisor_zero XReg (rv_or y_lo y_hi))
      (_ InstOutput (gen_trapif (IntCC.Equal) divisor_zero (zero_reg) (TrapCode.INTEGER_DIVISION_BY_ZERO)))
      (remainder ValueRegs (rem_i64_inst x_regs y_regs))
    )
    remainder))

;;;; Rules for `srem` i64 ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(rule 4 (lower (has_type $I64 (srem x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      (y_safe ValueRegs (safe_sdiv_divisor_i64 x_regs y_regs))
      (remainder ValueRegs (rem_i64_signed_inst x_regs y_safe))
    )
    remainder))
```

### Step 7: Add Helper for Safe Signed Divisor (i64)

File: `cranelift/codegen/src/isa/riscv32/lower.isle`

Add helper to check for safe signed division (similar to `safe_sdiv_divisor` but for i64):

```isle
(decl safe_sdiv_divisor_i64 (ValueRegs ValueRegs) ValueRegs)
(rule (safe_sdiv_divisor_i64 dividend divisor)
  (let (
      ;; Check divisor is not zero: (divisor_lo | divisor_hi) == 0
      (divisor_lo XReg (value_regs_get divisor 0))
      (divisor_hi XReg (value_regs_get divisor 1))
      (divisor_zero XReg (rv_or divisor_lo divisor_hi))
      (_ InstOutput (gen_trapif (IntCC.Equal) divisor_zero (zero_reg) (TrapCode.INTEGER_DIVISION_BY_ZERO)))
      
      ;; Check for INT_MIN / -1 overflow
      ;; INT_MIN = 0x80000000_00000000 (high=0x80000000, low=0x00000000)
      ;; -1 = 0xFFFFFFFF_FFFFFFFF (high=0xFFFFFFFF, low=0xFFFFFFFF)
      (dividend_lo XReg (value_regs_get dividend 0))
      (dividend_hi XReg (value_regs_get dividend 1))
      
      ;; Check dividend_hi == 0x80000000
      (int_min_hi XReg (imm $I32 0x80000000))
      (dividend_is_min_hi XReg (rv_xor dividend_hi int_min_hi))
      (dividend_is_min_hi_eq XReg (rv_seqz dividend_is_min_hi))
      
      ;; Check dividend_lo == 0x00000000
      (dividend_is_min_lo XReg (rv_seqz dividend_lo))
      
      ;; Check divisor_hi == 0xFFFFFFFF
      (neg_one_hi XReg (imm $I32 0xFFFFFFFF))
      (divisor_is_neg_one_hi XReg (rv_xor divisor_hi neg_one_hi))
      (divisor_is_neg_one_hi_eq XReg (rv_seqz divisor_is_neg_one_hi))
      
      ;; Check divisor_lo == 0xFFFFFFFF
      (neg_one_lo XReg (imm $I32 0xFFFFFFFF))
      (divisor_is_neg_one_lo XReg (rv_xor divisor_lo neg_one_lo))
      (divisor_is_neg_one_lo_eq XReg (rv_seqz divisor_is_neg_one_lo))
      
      ;; Combine checks: dividend == INT_MIN && divisor == -1
      (dividend_is_min XReg (rv_and dividend_is_min_hi_eq dividend_is_min_lo))
      (divisor_is_neg_one XReg (rv_and divisor_is_neg_one_hi_eq divisor_is_neg_one_lo_eq))
      (overflow_condition XReg (rv_and dividend_is_min divisor_is_neg_one))
      
      ;; Trap if overflow condition is true
      (_ InstOutput (gen_trapif (IntCC.NotEqual) overflow_condition (zero_reg) (TrapCode.INTEGER_OVERFLOW)))
    )
    divisor))
```

### Step 8: Implement Operand Tracking

File: `cranelift/codegen/src/isa/riscv32/inst/mod.rs`

Add operand tracking in `riscv32_get_operands` function for register allocation:

```rust
Inst::DivI64U { dividend, divisor, quotient, rem, counter } => {
    // Inputs: dividend and divisor register pairs
    for reg in dividend.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in divisor.regs_mut() {
        collector.reg_use(reg);
    }
    // Outputs: quotient register pair
    for reg in quotient.regs_mut() {
        collector.reg_early_def(reg);  // Use early_def to prevent overlap with inputs
    }
    // Temps: rem and counter (clobbered)
    for reg in rem.regs_mut() {
        collector.reg_early_def(reg);
    }
    collector.reg_early_def(counter);
}

Inst::DivI64S { dividend, divisor, quotient, rem, counter, sign } => {
    // Similar to DivI64U, plus sign temp
    for reg in dividend.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in divisor.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in quotient.regs_mut() {
        collector.reg_early_def(reg);
    }
    for reg in rem.regs_mut() {
        collector.reg_early_def(reg);
    }
    collector.reg_early_def(counter);
    collector.reg_early_def(sign);
}

Inst::RemI64U { dividend, divisor, remainder, quotient, counter } => {
    for reg in dividend.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in divisor.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in remainder.regs_mut() {
        collector.reg_early_def(reg);
    }
    for reg in quotient.regs_mut() {
        collector.reg_early_def(reg);
    }
    collector.reg_early_def(counter);
}

Inst::RemI64S { dividend, divisor, remainder, quotient, counter, sign } => {
    for reg in dividend.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in divisor.regs_mut() {
        collector.reg_use(reg);
    }
    for reg in remainder.regs_mut() {
        collector.reg_early_def(reg);
    }
    for reg in quotient.regs_mut() {
        collector.reg_early_def(reg);
    }
    collector.reg_early_def(counter);
    collector.reg_early_def(sign);
}
```

**Note**: Use `regs_mut()` to get mutable access for the collector. Use `reg_early_def()` for temp registers to prevent them from overlapping with input registers.

### Step 9: Implement Worst-Case Size Handling

File: `cranelift/codegen/src/isa/riscv32/inst/emit.rs`

The division instruction will likely exceed the current `worst_case_size()` of 84 bytes. We have two options:

**Option A**: Emit an island (like `BrTable` does):

- Check if island is needed before emitting
- Emit jump around island if needed
- Emit island
- Emit actual division code

**Option B**: Exclude from worst-case size check:

- Add `DivI64U`, `DivI64S`, `RemI64U`, `RemI64S` to the `emits_own_island` match in `emit.rs`
- This allows them to exceed worst-case size

**Recommendation**: Option B is simpler. The division code will be large but deterministic.

Add to the `emits_own_island` match in `emit.rs`:

```rust
let emits_own_island = match self {
    Inst::BrTable { .. }
    | Inst::ReturnCall { .. }
    | Inst::ReturnCallInd { .. }
    | Inst::Call { .. }
    | Inst::CallInd { .. }
    | Inst::EmitIsland { .. }
    | Inst::DivI64U { .. }
    | Inst::DivI64S { .. }
    | Inst::RemI64U { .. }
    | Inst::RemI64S { .. } => true,
    _ => false,
};
```

### Step 10: Implement Display/Debug for New Instructions

File: `cranelift/codegen/src/isa/riscv32/inst/mod.rs`

Add pretty-printing support for the new instructions in the `Display` implementation. Use the `format_regs` helper pattern from existing code:

```rust
Inst::DivI64U { dividend, divisor, quotient, .. } => {
    let format_regs = |regs: &[Reg]| -> String {
        let mut x = if regs.len() > 1 {
            String::from("[")
        } else {
            String::default()
        };
        regs.iter().for_each(|i| {
            x.push_str(reg_name(*i).as_str());
            if *i != *regs.last().unwrap() {
                x.push_str(",");
            }
        });
        if regs.len() > 1 {
            x.push_str("]");
        }
        x
    };
    write!(f, "div.i64u {}, {} -> {}", 
           format_regs(dividend.regs()),
           format_regs(divisor.regs()),
           format_regs(quotient.regs()))
}

Inst::DivI64S { dividend, divisor, quotient, .. } => {
    // Similar to DivI64U
    write!(f, "div.i64s {}, {} -> {}", 
           format_regs(dividend.regs()),
           format_regs(divisor.regs()),
           format_regs(quotient.regs()))
}

Inst::RemI64U { dividend, divisor, remainder, .. } => {
    write!(f, "rem.i64u {}, {} -> {}", 
           format_regs(dividend.regs()),
           format_regs(divisor.regs()),
           format_regs(remainder.regs()))
}

Inst::RemI64S { dividend, divisor, remainder, .. } => {
    write!(f, "rem.i64s {}, {} -> {}", 
           format_regs(dividend.regs()),
           format_regs(divisor.regs()),
           format_regs(remainder.regs()))
}
```

**Note on Emulator Support**: The emulator does NOT need to be updated. `DivI64U`, `DivI64S`, `RemI64U`, and `RemI64S` are Cranelift MInst variants (internal machine instructions), not actual RISC-V32 hardware instructions. When we emit these MInst variants in `emit.rs`, they get expanded into actual RISC-V32 instructions (add, sub, shifts, branches, etc.) that the emulator already knows how to execute. The emulator will automatically work correctly once the emission code is implemented.

## Testing Strategy

### Test Files Created

1. **Runtest**: `cranelift/filetests/filetests/runtests/i64-div.clif`

   - Functional tests for i64 division/remainder operations
   - Tests across multiple targets (aarch64, riscv32, riscv64, x86_64, etc.)
   - Includes test cases for:
     - Basic unsigned/signed division (`udiv_i64`, `sdiv_i64`)
     - Power-of-2 divisors (`udiv_pow2_i64`, `sdiv_pow2_i64`)
     - Constant divisors (`udiv_by_const_1337_i64`, `sdiv_by_const_1337_i64`)
     - Negative divisors (`sdiv_neg_pow2_i64`, `sdiv_by_const_neg_1337_i64`)
     - Remainder operations (`urem_i64`, `srem_i64`)
     - Immediate remainder (`urem_imm_i64`, `srem_imm_i64`)
     - Edge cases: large values, negative values, zero

2. **ISA Test**: `cranelift/filetests/filetests/isa/riscv32/i64-div.clif`

   - Code generation verification for riscv32-specific i64 division
   - Uses `test compile precise-output` to verify generated VCode
   - Documents expected structure:
     - Division by zero checks
     - Register initialization (remainder, quotient, counter)
     - Binary long division loop structure
     - Sign handling for signed operations
     - Return value (quotient for div, remainder for rem)

### Running Tests

1. **Run functional tests**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/i64-div.clif
   ```

2. **Run ISA-specific codegen tests**:
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/isa/riscv32/i64-div.clif
   ```

3. **Test existing division tests** (should still pass):
   ```bash
   cargo run --bin clif-util -- test filetests/filetests/runtests/urem.clif
   cargo run --bin clif-util -- test filetests/filetests/runtests/div-checks.clif
   ```


**Note**: The emulator does not need updates. When Cranelift emits `DivI64U`/etc. MInst variants, they are expanded into actual RISC-V32 instructions (add, sub, shifts, branches, etc.) that the emulator already handles. Tests will automatically use the emulator when running riscv32 tests.

## Additional Considerations

### Temp Register Allocation

Temp registers are allocated during lowering in ISLE using:

- `temp_writable_reg` for single registers (e.g., counter, sign)
- `temp_value_regs $I64` for register pairs (e.g., quotient, remainder)

The register allocator will assign actual physical registers. All temp registers should be `RegClass::Int` (XReg).

### Register Constraints

- All registers must be integer registers (XReg)
- Register pairs are stored in machine-endian order: `parts[0]` = low 32 bits, `parts[1]` = high 32 bits
- Access via: `value_regs.regs()[0]` and `value_regs.regs()[1]`
- Convert to XReg: `XReg::new(reg).unwrap()` for read-only registers
- Convert to WritableXReg: `writable_xreg_new(Writable::from_reg(reg))` for writable registers
- Zero register: Use `zero_reg()` or `writable_zero_reg()` (x0, always zero)

### Code Size Considerations

The binary long division algorithm will generate a significant amount of code:

- 64 iterations of the loop
- Each iteration: shift (~5 instructions), compare (~3 instructions), conditional subtract (~5 instructions), set bit (~2 instructions)
- Estimated: ~15 instructions per iteration × 64 = ~960 instructions
- At 4 bytes per instruction = ~3840 bytes
- This will definitely exceed `worst_case_size()` of 84 bytes

**Solution**: Exclude division instructions from worst-case size checks (add to `emits_own_island` match in `emit.rs`).

Alternatively, we could emit an island like `BrTable` does, but since the size is deterministic and known, excluding from the check is simpler.

### Optimization Opportunities (Future)

1. **Power-of-2 divisors**: Use shift instead of division (can be detected in lowering)
2. **Small divisors**: Use multiplication-based approach
3. **32-bit divisor**: Optimized path (simpler algorithm - divide high, combine, divide low)
4. **Constant divisors**: Special-case handling in lowering rules
5. **Early exit**: If remainder < divisor, skip iterations

### Instruction Operand Requirements

The MInst variants need to specify:

- **Inputs**: dividend (ValueRegs), divisor (ValueRegs)
- **Outputs**: quotient (ValueRegs) or remainder (ValueRegs)
- **Temps**: rem (ValueRegs), counter (WritableReg), sign (WritableReg for signed ops)

These will be used by the register allocator to:

- Track register uses/defs (via `riscv32_get_operands`)
- Allocate physical registers
- Handle register spills if needed

**Important**: The register allocator needs to know:

- Which registers are **used** (inputs): `collector.reg_use(reg)`
- Which registers are **defined** (outputs): `collector.reg_def(reg)` or `collector.reg_early_def(reg)`
- Use `reg_early_def` for temps to prevent them from overlapping with input registers

## Implementation Checklist Summary

### Core Implementation

- [ ] Add MInst enum variants (DivI64U, DivI64S, RemI64U, RemI64S)
- [ ] Add ISLE helper declarations
- [ ] Implement binary long division algorithm in Rust (emit.rs)
- [ ] Implement signed division (handle signs, overflow)
- [ ] Implement remainder operations

### Supporting Infrastructure

- [ ] Implement 64-bit left shift helper
- [ ] Implement 64-bit negation helper (for signed division)
- [ ] Add operand tracking (riscv32_get_operands)
- [ ] Handle worst-case size (exclude from check)
- [ ] Implement Display/Debug for new instructions
- [ ] Add lowering rules (udiv/sdiv/urem/srem i64)
- [ ] Add safe_sdiv_divisor_i64 helper

### Testing & Validation

- [ ] Test with filetests (i64-div.clif runtest and isa/riscv32/i64-div.clif ISA test)
- [ ] Test existing division tests (urem.clif, div-checks.clif)
- [ ] Verify edge cases (division by zero, INT_MIN / -1)
- [ ] Verify emulator execution (automatic - emulator handles emitted RISC-V32 instructions)

## Success Criteria

- All i64 division/remainder tests pass
- No performance regression for 32-bit operations
- Correct handling of edge cases (division by zero, overflow)
- Emulator correctly executes generated code (automatic - emulator handles emitted RISC-V32 instructions)
- Code compiles without warnings
- Worst-case size handling doesn't break code generation
- Register allocation works correctly (no register conflicts)

## References

- **RISC-V ISA specification**: `/Users/yona/dev/photomancer/riscv-isadoc/source/rvm.adoc`
- **Existing i64 helpers**: `cranelift/codegen/src/isa/riscv32/inst.isle` (see `sub_i64` for register pair pattern)
- **x64 AtomicRmwSeq example**: `cranelift/codegen/src/isa/x64/inst/emit.rs` (lines 1435-1546) - complex multi-instruction sequence with loops
- **riscv32 BrTable example**: `cranelift/codegen/src/isa/riscv32/inst/emit.rs` (lines 1373-1524) - complex multi-instruction sequence
- **riscv32 return_call example**: `cranelift/codegen/src/isa/riscv32/inst/emit.rs` (lines 2775-2806) - helper function pattern
- **x64 division implementation**: `cranelift/codegen/src/isa/x64/lower.isle` (lines 4349-4441) - hardware-supported division
- **Existing plan**: `lightplayer/plans/fix-filetests/i64-division-plan.md`

