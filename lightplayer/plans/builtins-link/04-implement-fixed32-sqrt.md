# Phase 4: Implement `fixed32/sqrt.rs`

## Goal

Implement `__lp_fixed32_sqrt` function using Newton-Raphson method to avoid 64-bit types in the compiler.

## Steps

### 4.1 Create function file

- Create `src/fixed32/sqrt.rs`
- Add `#[no_mangle] pub extern "C" fn __lp_fixed32_sqrt(x: i32) -> i32`
- Handle edge cases: return 0 for x <= 0

### 4.2 Implement algorithm

- Use Newton-Raphson method with reciprocal multiplication
- Scale input: `x_scaled = x << 16` (use i64 internally)
- Initial guess: `max(x_scaled >> 9, 1)`
- Iterate: `guess = (guess + x_scaled / guess) >> 1`
- Use reciprocal multiplication for division step (avoid i64 division)
- Result: `guess >> 8` (truncate to i32)

### 4.3 Handle precision

- Use sufficient iterations (6-16) for good precision
- Ensure convergence
- Handle edge cases (small values, large values)

### 4.4 Add to module

- Export function in `src/fixed32/mod.rs`

## Algorithm Reference

Based on `lightplayer/crates/lp-glsl-builtins-src/src/builtins/fixed32/sqrt_recip.rs`:
- Newton-Raphson: `guess = (guess + x_scaled / guess) >> 1`
- Use reciprocal multiplication for `x_scaled / guess`
- Scale appropriately for fixed16x16 format

## Files to Create

- `lightplayer/crates/lp-builtins/src/fixed32/sqrt.rs`

## Files to Modify

- `lightplayer/crates/lp-builtins/src/fixed32/mod.rs` (add `mod sqrt;`)

## Success Criteria

- Function compiles for both native and riscv32 targets
- Function signature: `(i32) -> i32`
- Function handles edge cases (x <= 0 returns 0)
- Function is exported with `#[no_mangle] pub extern "C"`

## Notes

- Can use i64 internally for calculations
- Must use reciprocal multiplication for division (no i64 division)
- Precision should be reasonable for fixed16x16 format

