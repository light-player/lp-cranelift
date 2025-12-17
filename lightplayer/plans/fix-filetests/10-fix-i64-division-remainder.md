# Phase 10: Fix i64 Division and Remainder Operations

## Goal

Implement i64 division (`udiv.i64`, `sdiv.i64`) and remainder (`urem.i64`, `srem.i64`) operations for riscv32 to fix 5 failing tests.

## Prerequisites

- Phase 3 completed: Basic i64 handling works (add, sub, mul, shifts, etc.)
- Phase 4 completed: ISLE panics fixed

## Problem Analysis

### Current Status

RISC-V32 only provides native 32-bit division instructions (`div`, `divu`, `rem`, `remu`). The ISLE lowering rules for i64 division/remainder are commented out:

```isle
;; I64 division not yet implemented for riscv32 (needs complex register pair arithmetic)
;; (rule 2 (lower (has_type $I64 (udiv x y)))
;;   ...)
```

### Affected Tests

1. **`arithmetic.clif`** - Fails with: `Unsupported feature: should be implemented in ISLE: inst = v2 = udiv.i64 v0, v1`
2. **`udiv.clif`** - Fails with: `Unsupported feature: should be implemented in ISLE: inst = v2 = udiv.i64 v0, v1`
3. **`div-checks.clif`** - Fails with: `Unsupported feature: should be implemented in ISLE: inst = v2 = srem.i64 v0, v1`
4. **`srem_opts.clif`** - Fails with: `Unsupported feature: should be implemented in ISLE: inst = v2 = srem.i64 v0, v1`
5. **`urem.clif`** - Fails with: `Unsupported feature: should be implemented in ISLE: inst = v2 = urem.i64 v0, v1`

### Reference Implementation

There's already a detailed plan at `lightplayer/plans/rv32-64bit-div.md` that outlines the full implementation strategy. This phase should follow that plan.

## Implementation Strategy

**Decision**: Implement in Rust (not ISLE) following the pattern used by other complex multi-instruction sequences (e.g., `AtomicRmwSeq`, `BrTable`).

### Algorithm: Binary Long Division

For arbitrary 64-bit divisors, use binary long division (restoring division):

1. Initialize: `quotient = 0`, `remainder = dividend`
2. For each bit position (63 down to 0):
   - Shift `remainder` left by 1 bit
   - Subtract `divisor` from `remainder`
   - If result is non-negative: set quotient bit, keep new remainder
   - If result is negative: clear quotient bit, restore old remainder
3. Handle signs for signed division
4. Handle division by zero

### Optimizations

1. **32-bit divisor optimization**: If divisor high 32 bits are zero, use simpler algorithm
2. **Power-of-2 optimization**: Use shifts for powers of 2
3. **Small divisor optimization**: Use native 32-bit division when possible

## Implementation Plan

### Step 1: Add MInst Variants

**File**: `cranelift/codegen/src/isa/riscv32/inst.isle`

Add new instruction variants:

- `DivI64U` - Unsigned i64 division
- `DivI64S` - Signed i64 division
- `RemI64U` - Unsigned i64 remainder
- `RemI64S` - Signed i64 remainder

Each needs:

- Dividend register pair (ValueRegs)
- Divisor register pair (ValueRegs)
- Quotient/remainder output (ValueRegs)
- Temporary registers for algorithm (counter, temp values)

### Step 2: Add ISLE Helper Declarations

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

Add helper function declarations:

```isle
(decl div_i64_inst (ValueRegs ValueRegs) ValueRegs)
(decl div_i64_signed_inst (ValueRegs ValueRegs) ValueRegs)
(decl rem_i64_inst (ValueRegs ValueRegs) ValueRegs)
(decl rem_i64_signed_inst (ValueRegs ValueRegs) ValueRegs)
```

### Step 3: Implement Binary Long Division in Rust

**File**: `cranelift/codegen/src/isa/riscv32/inst/emit.rs`

Add implementation in `emit_uncompressed` function:

```rust
&Inst::DivI64U {
    dividend,
    divisor,
    quotient,
    rem,
    counter,
} => {
    // Extract register pairs
    let dividend_regs = dividend.regs();
    let divisor_regs = divisor.regs();
    let quotient_regs = quotient.regs();
    let rem_regs = rem.regs();

    // Check for division by zero
    // Emit trap if divisor == 0

    // Initialize: remainder = dividend, quotient = 0

    // Binary long division loop (64 iterations)
    // For each bit position:
    //   - Shift remainder left
    //   - Try subtracting divisor
    //   - Update quotient and remainder

    // Emit all instructions
}
```

**Key considerations**:

- Use temporary registers for algorithm state
- Handle register pair operations correctly
- Emit proper RISC-V instructions
- Handle worst_case_size() for instruction sequences

### Step 4: Implement Signed Division

Similar to unsigned, but:

- Handle sign extension
- Handle overflow cases (e.g., INT64_MIN / -1)
- Convert to unsigned, perform division, adjust signs

### Step 5: Implement Remainder Operations

Can reuse division algorithm - remainder is the final remainder value after division.

### Step 6: Add Lowering Rules

**File**: `cranelift/codegen/src/isa/riscv32/lower.isle`

Uncomment and implement the lowering rules:

```isle
(rule (lower (has_type $I64 (udiv x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      (quotient ValueRegs (div_i64_inst x_regs y_regs))
    )
    quotient))

(rule (lower (has_type $I64 (sdiv x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      (quotient ValueRegs (div_i64_signed_inst x_regs y_regs))
    )
    quotient))

(rule (lower (has_type $I64 (urem x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      (remainder ValueRegs (rem_i64_inst x_regs y_regs))
    )
    remainder))

(rule (lower (has_type $I64 (srem x y)))
  (if-let true (has_m))
  (let (
      (x_regs ValueRegs x)
      (y_regs ValueRegs y)
      (remainder ValueRegs (rem_i64_signed_inst x_regs y_regs))
    )
    remainder))
```

### Step 7: Add Operand Tracking

**File**: `cranelift/codegen/src/isa/riscv32/inst.rs`

Add operand tracking in `riscv32_get_operands`:

- `reg_use` for dividend and divisor inputs
- `reg_early_def` for quotient/remainder outputs
- `reg_use` for temporary registers

### Step 8: Handle worst_case_size()

**File**: `cranelift/codegen/src/isa/riscv32/inst.rs`

Either:

- Emit as an "island" (separate code block)
- Exclude from worst_case_size check
- Provide accurate size estimate

### Step 9: Implement Display/Debug

**File**: `cranelift/codegen/src/isa/riscv32/inst.rs`

Add `Display` and `Debug` implementations for new instruction variants.

## Testing

After implementation:

```bash
# Test individual operations
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/udiv.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/urem.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/div-checks.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/srem_opts.clif
cargo run --package cranelift-tools --bin clif-util -- test cranelift/filetests/filetests/runtests/arithmetic.clif

# Test all riscv32 tests
cargo run --package cranelift-tools --bin clif-util -- test $(find cranelift/filetests/filetests/runtests -name "*.clif" -exec grep -l "^target riscv32" {} \;)
```

## Success Criteria

- ✅ All 5 affected tests pass
- ✅ No "Unsupported feature" errors for i64 division/remainder
- ✅ Division by zero traps correctly
- ✅ Signed division handles overflow correctly
- ✅ Performance is acceptable (optimizations help)
- ✅ Code follows existing patterns (AtomicRmwSeq, BrTable)

## Estimated Time

- Step 1-2: 1 hour (MInst variants, ISLE declarations)
- Step 3: 4-6 hours (Binary long division implementation)
- Step 4: 2-3 hours (Signed division)
- Step 5: 1-2 hours (Remainder operations)
- Step 6-9: 2-3 hours (Lowering rules, operand tracking, etc.)

**Total**: 10-15 hours

## Related Files

- `lightplayer/plans/rv32-64bit-div.md` - Detailed implementation plan
- `cranelift/codegen/src/isa/riscv32/lower.isle` - Lowering rules (lines 900-1050)
- `cranelift/codegen/src/isa/riscv32/inst/emit.rs` - Instruction emission
- `cranelift/codegen/src/isa/riscv32/inst.isle` - Instruction definitions

## Notes

- This is a complex implementation but follows established patterns
- The binary long division algorithm is well-understood
- Optimizations can be added incrementally
- Consider testing with edge cases (INT64_MIN, INT64_MAX, powers of 2, etc.)

