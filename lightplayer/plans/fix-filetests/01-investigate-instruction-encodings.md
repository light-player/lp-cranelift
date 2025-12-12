# Phase 1: Investigate Instruction Encodings - COMPLETED

## Summary

Investigation revealed that the failing tests are due to **Cranelift's riscv32 backend incorrectly handling i64 operations**. The riscv32 backend was copied from riscv64 but not properly adapted for 32-bit operation.

### Root Cause Analysis

- **RV32I Reality**: 32-bit registers (XLEN=32), no native 64-bit integer support
- **Cranelift Design**: riscv32 backend implements i64 using register pairs (low + high 32-bit registers)
- **Missing Rule**: Plain `uextend.i64` operations lack a lowering rule, causing fallback to incorrect single-instruction generation
- **Example**: `uextend.i64` should produce (0, value) in register pair, but generates `SRLI a1, a5, 32` (now correctly decoded) instead

### Key Findings

1. riscv32 backend has incorrect comments (still references "64-bit ISA")
2. i64 operations should either be rejected or properly legalized for 32-bit registers
3. Current implementation generates wrong instructions that execute but produce wrong results

## Goal

Investigate instruction encoding issues between Cranelift and the emulator. Initial hypothesis was that the emulator didn't recognize Cranelift's instruction encodings, but investigation revealed that Cranelift generates incorrect instructions for riscv32.

## Key Finding

**Cranelift's riscv32 backend has bugs in i64 handling**. It generates single 32-bit instructions for operations that should use register pairs on riscv32. The failing tests are not due to emulator decoding issues, but due to Cranelift generating wrong code.

Example: `uextend.i64` from i32 generates `SRLI a1, a5, 32` instead of proper i64 zero-extend logic.

## Affected Test Files

These tests fail with "Unknown I-type instruction" errors:

```bash
# Run individual tests to see the exact instruction bytes:
cargo run --bin clif-util -- test filetests/filetests/runtests/arithmetic-extends.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/udiv.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/shifts.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/smulhi.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/extend.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/umulhi.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/rotl.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/fibonacci.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/popcnt.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/br_table.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/uadd_overflow_trap.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/icmp.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/rotr.clif
cargo run --bin clif-util -- test filetests/filetests/runtests/bitrev.clif
```

## Investigation Steps

1. **Extract instruction bytes from failing tests**:

   - `0x0207d593` decodes to `SRLI a1, a5, 32` (not `SRLI a1, a0, 32` as initially thought)
   - This appears in `arithmetic-extends.clif` test for `uextend.i64` operation

2. **Root Cause Analysis**:

   - riscv32 has 32-bit registers, so i64 operations require register pairs
   - `uextend.i64` from i32 should be a no-op (32-bit register already zero-extended)
   - But Cranelift generates `SRLI a1, a5, 32` which is incorrect for this operation
   - **Conclusion**: Cranelift's riscv32 backend has incomplete/broken i64 support

3. **Impact**:

   - Tests fail because Cranelift generates wrong instructions for i64 operations on riscv32
   - The emulator correctly decodes the instructions, but they produce wrong results
   - This is a Cranelift bug, not an emulator decoding issue

## Key Files to Examine

- `cranelift/codegen/src/isa/riscv32/inst/args.rs` - Check `option_funct12()` and `funct3()` methods
- `cranelift/codegen/src/isa/riscv32/inst/encode.rs` - See how instructions are encoded
- `lightplayer/crates/lp-riscv-tools/src/decode.rs` - Current decode logic (lines 170-260)
- `lightplayer/crates/lp-riscv-tools/src/encode.rs` - Emulator's encoding functions

## Expected Outcome

After this phase, we understand:

- The issue is **not** emulator decoding problems
- Cranelift's riscv32 backend generates incorrect instructions for i64 operations
- riscv32 i64 support in Cranelift is incomplete/broken

## Required Fixes for riscv32 Backend

### 1. Update Incorrect Comments and Documentation

- Change `//! risc-v 64-bit Instruction Set Architecture.` to `//! risc-v 32-bit Instruction Set Architecture.`
- Remove all RV64 references that don't apply to RV32
- Update file header in `lower.isle`: `;; riscv64 instruction selection` → `;; riscv32 instruction selection`

### 2. Add Missing uextend.i64 Lowering Rule

The riscv32 backend already implements i64 using register pairs, but is missing a rule for plain `uextend.i64` operations.

**Required Rule**: For `uextend.i64` from i32, generate register pair:

- Low register: copy of the i32 value
- High register: 0 (since zero-extend fills upper 32 bits with zeros)

**ISLE Rule Example**:

```
(rule (lower (has_type $I64 (uextend x @ (value_type $I32))))
  (value_regs x (imm $I32 0)))
```

**Current Workaround**: Falls back to incorrect single-instruction generation.

### 3. Verify Complete i64 Support

Check that all i64 operations have proper lowering rules:

- ✅ `iconst` (splits into low/high parts)
- ✅ `iadd` (uses carry propagation)
- ❌ `uextend` (missing rule)
- Check `isub`, `imul`, shifts, comparisons, etc.

### 4. Update Test Expectations

- With proper `uextend.i64` rule, tests should pass
- Current state: `uextend.i64` generates wrong instruction, causing wrong results
- Other i64 operations may have similar issues

## Phase 1 Resolution

✅ **Emulator decoding fixed**: Added support for Cranelift's SRLI encoding (funct6=0x0)
✅ **Root cause identified**: riscv32 backend missing `uextend.i64` lowering rule
✅ **Architecture validated**: riscv32 correctly uses register pairs for i64 operations

## Next Phase

Proceed to Phase 3 to implement the missing `uextend.i64` lowering rule in `cranelift/codegen/src/isa/riscv32/lower.isle`. The riscv32 backend architecture is sound, but incomplete.
