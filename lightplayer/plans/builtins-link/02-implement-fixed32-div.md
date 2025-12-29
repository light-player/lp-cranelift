# Phase 2: Implement `fixed32/div.rs`

## Goal

Implement `__lp_fixed32_div` function using reciprocal multiplication to avoid 64-bit types in the compiler.

## Steps

### 2.1 Create function file

- Create `src/fixed32/div.rs`
- Add `#[no_mangle] pub extern "C" fn __lp_fixed32_div(dividend: i32, divisor: i32) -> i32`
- Use reciprocal multiplication algorithm (similar to existing `div_recip.rs` reference)

### 2.2 Implement algorithm

- Handle division by zero (saturate to max/min based on sign)
- Compute reciprocal: `recip = 0x8000_0000 / divisor` (u32 division)
- Calculate quotient: `(dividend * recip * 2) >> 16` using only 32-bit operations
- Handle sign correctly (XOR of dividend and divisor signs)

### 2.3 Add to module

- Export function in `src/fixed32/mod.rs`
- Re-export in `src/lib.rs` if needed

### 2.4 Verify compilation

- Ensure function compiles for both native and `riscv32imac-unknown-none-elf` targets

## Algorithm Reference

Based on `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/reference/div_recip.rs`:
- Use reciprocal multiplication: `quotient = (dividend * recip * 2) >> 16`
- Reciprocal: `recip = 0x8000_0000 / divisor`
- Handle sign by computing absolute values, then applying sign

## Files to Create

- `lightplayer/crates/lp-builtins/src/fixed32/div.rs`

## Files to Modify

- `lightplayer/crates/lp-builtins/src/fixed32/mod.rs` (add `mod div;`)
- `lightplayer/crates/lp-builtins/src/lib.rs` (export if needed)

## Success Criteria

- Function compiles for both native and riscv32 targets
- Function signature matches expected: `(i32, i32) -> i32`
- Function is exported with `#[no_mangle] pub extern "C"`

## Notes

- Must use only 32-bit operations (no i64/u64 in function signature or intermediate values that leak to compiler)
- Can use i64 internally for calculations, but result must be i32
- Handle edge cases: division by zero, overflow, sign handling

