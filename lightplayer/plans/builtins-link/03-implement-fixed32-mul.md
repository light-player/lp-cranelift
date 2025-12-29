# Phase 3: Implement `fixed32/mul.rs`

## Goal

Implement `__lp_fixed32_mul` function handling overflow/saturation to avoid 64-bit types in the compiler.

## Steps

### 3.1 Create function file

- Create `src/fixed32/mul.rs`
- Add `#[no_mangle] pub extern "C" fn __lp_fixed32_mul(a: i32, b: i32) -> i32`
- Fixed-point multiplication: `(a * b) >> 16`

### 3.2 Implement algorithm

- Use i64 internally for multiplication to avoid overflow
- Multiply: `result_wide = (a as i64) * (b as i64)`
- Right shift: `result_wide >> 16`
- Saturate to i32 range: clamp to `[i32::MIN, 0x7FFF_0000]` (max fixed-point value)
- Return as i32

### 3.3 Handle edge cases

- Overflow: saturate to max/min fixed-point values
- Underflow: saturate to min fixed-point value
- Zero handling: return 0 if either operand is 0

### 3.4 Add to module

- Export function in `src/fixed32/mod.rs`

## Algorithm Reference

Based on `lightplayer/crates/lp-glsl/src/backend/transform/fixed32/converters/arithmetic.rs`:
- Fixed-point multiplication: `(a * b) >> shift_amount`
- Use i64 intermediate to avoid overflow
- Saturate before truncation to i32

## Files to Create

- `lightplayer/crates/lp-builtins/src/fixed32/mul.rs`

## Files to Modify

- `lightplayer/crates/lp-builtins/src/fixed32/mod.rs` (add `mod mul;`)

## Success Criteria

- Function compiles for both native and riscv32 targets
- Function signature: `(i32, i32) -> i32`
- Function handles overflow/underflow correctly (saturates)
- Function is exported with `#[no_mangle] pub extern "C"`

## Notes

- Can use i64 internally (Rust handles this), but compiler won't see 64-bit operations
- Must saturate correctly to fixed-point range
- Max fixed-point value: `0x7FFF_0000` (not `i32::MAX`)

