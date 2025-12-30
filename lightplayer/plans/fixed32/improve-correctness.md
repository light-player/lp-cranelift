# Hybrid Infinity/NaN Support for Fixed32

## Overview

Implement sentinel value encoding for infinity and NaN in fixed32 format, preserving IEEE semantics for comparisons, division, and math functions while keeping basic arithmetic operations fast with saturation.

## Rationale

For GLSL shaders, the most pragmatic approach balances correctness where it matters most with performance for hot paths:

- **Comparisons**: Critical for correctness - many shaders rely on conditional logic
- **Division**: Common operation where division by zero producing infinity is important
- **Math functions**: Less common but when used, proper infinity/NaN handling matters
- **Basic arithmetic**: Performance-critical path where overflow is rare - use saturation

This hybrid approach preserves semantics where correctness matters most while keeping the hot path (add/sub/mul) fast.

## Encoding Scheme

Reserve sentinel values in i32 (Fixed16x16 format):

```
Normal range:    0x80000000 to 0x7FFF0000 (representable fixed-point)
Sentinel range:  0x7FFF0001 to 0x7FFFFFFF (positive special values)
                 0x80000001 to 0x80000000 (negative special values)

Specific sentinels:
  +Inf:  0x7FFF0001
  -Inf:  0x80000001  
  +NaN:  0x7FFF0002
  -NaN:  0x80000002
```

## Implementation Tasks

### 1. Core Infrastructure (`lightplayer/crates/lp-glsl/src/ir_utils/fixed_point.rs`)

Add helper functions for sentinel value detection and creation:

- `is_inf_fixed(value: i32) -> bool` - Check if value is infinity
- `is_nan_fixed(value: i32) -> bool` - Check if value is NaN
- `is_finite_fixed(value: i32) -> bool` - Check if value is finite
- `is_special_fixed(value: i32) -> bool` - Check if value is any special value
- `create_inf_fixed(builder, format, positive: bool) -> Value` - Create infinity constant
- `create_nan_fixed(builder, format) -> Value` - Create NaN constant
- `create_special_fixed_constants(builder, format)` - Helper to create all special constants

### 2. Constant Conversion (`lightplayer/crates/lp-glsl/src/transform/fixed32/converters/constants.rs`)

Update `convert_f32const`:
- Detect infinity/NaN before clamping in `float_to_fixed16x16`
- Map `f32::INFINITY` → `0x7FFF0001`, `f32::NEG_INFINITY` → `0x80000001`
- Map `f32::NAN` → `0x7FFF0002` (canonical NaN)
- Update `float_to_fixed16x16` in `types.rs` to handle special values

### 3. Comparison Operations (`lightplayer/crates/lp-glsl/src/transform/fixed32/converters/comparison.rs`)

Update `convert_fcmp`:
- Check both operands for NaN before comparison
- Implement NaN semantics: `NaN != NaN`, all comparisons with NaN return false
- Handle infinity comparisons correctly (e.g., `+Inf > finite`, `-Inf < finite`)
- Preserve Ordered/Unordered semantics properly

Update `convert_fmax` and `convert_fmin`:
- Handle NaN inputs (return the non-NaN operand, or NaN if both are NaN)
- Handle infinity inputs correctly

### 4. Division (`lightplayer/crates/lp-glsl/src/transform/fixed32/converters/arithmetic.rs`)

Update `convert_fdiv`:
- Check numerator and denominator for infinity/NaN before division
- Implement IEEE rules:
  - `finite / 0` → `±Inf` (based on sign)
  - `0 / 0` → `NaN`
  - `Inf / finite` → `±Inf`
  - `Inf / Inf` → `NaN`
  - `finite / Inf` → `0`
  - Any operation with NaN → `NaN`
- Keep existing saturation logic as fallback for non-special cases

### 5. Math Functions (`lightplayer/crates/lp-glsl/src/transform/fixed32/converters/math.rs`)

Update `convert_sqrt`:
- Check for infinity input → return infinity
- Check for NaN input → return NaN
- Check for negative input → return NaN (already handled, but verify with sentinel check)

Update `convert_ceil` and `convert_floor`:
- Check for infinity/NaN inputs → preserve them
- Only apply rounding logic to finite values

### 6. Basic Arithmetic (`lightplayer/crates/lp-glsl/src/transform/fixed32/converters/arithmetic.rs`)

Keep `convert_fadd`, `convert_fsub`, `convert_fmul` unchanged:
- These use saturation/wrapping for overflow (performance-critical)
- No infinity/NaN propagation in arithmetic (by design for performance)

Update `convert_fneg`:
- Check for infinity/NaN → preserve with flipped sign bit
- Only negate finite values

Update `convert_fabs`:
- Check for infinity/NaN → preserve (abs of NaN is NaN, abs of Inf is Inf)
- Only apply abs logic to finite values

### 7. Type Conversion (`lightplayer/crates/lp-glsl/src/transform/fixed32/types.rs`)

Update `float_to_fixed16x16`:
- Check for `f32::INFINITY`, `f32::NEG_INFINITY`, `f32::NAN` before clamping
- Return sentinel values for special cases
- Only clamp finite values

## Testing Strategy

### Unit Tests (`lightplayer/crates/lp-glsl/tests/runtime_fixed_point.rs`)

Add comprehensive tests:

**Infinity Tests:**
- `test_infinity_constant` - Verify +Inf/-Inf constants are preserved
- `test_division_by_zero_produces_infinity` - Verify finite/0 → Inf
- `test_infinity_comparisons` - Test all comparison operators with infinity
- `test_infinity_arithmetic` - Verify infinity propagation in division
- `test_infinity_math_functions` - Test sqrt/ceil/floor with infinity

**NaN Tests:**
- `test_nan_constant` - Verify NaN constant is preserved
- `test_zero_over_zero_produces_nan` - Verify 0/0 → NaN
- `test_nan_comparisons` - Verify NaN != NaN, NaN comparisons return false
- `test_nan_propagation` - Verify NaN propagates through operations
- `test_nan_math_functions` - Test sqrt/ceil/floor with NaN

**Edge Cases:**
- `test_infinity_over_infinity` - Should produce NaN
- `test_finite_over_infinity` - Should produce 0
- `test_sqrt_negative` - Should produce NaN
- `test_sqrt_infinity` - Should produce infinity
- `test_neg_infinity` - Should preserve infinity with flipped sign
- `test_abs_infinity` - Should preserve infinity
- `test_abs_nan` - Should preserve NaN

**Regression Tests:**
- Verify existing tests still pass (finite value behavior unchanged)
- Test that arithmetic operations (add/sub/mul) still use saturation

### Filetests (`lightplayer/crates/lp-glsl-filetests/filetests/`)

Add filetests for:
- `infinity-comparison.glsl` - Test infinity comparisons
- `nan-comparison.glsl` - Test NaN comparisons  
- `infinity-division.glsl` - Test division with infinity
- `nan-division.glsl` - Test division producing NaN
- `infinity-math.glsl` - Test math functions with infinity
- `nan-math.glsl` - Test math functions with NaN

## Performance Considerations

- Sentinel checks use simple integer comparisons (fast)
- Use `select` instructions instead of branches (better for pipelining)
- Arithmetic operations (add/sub/mul) remain unchanged (no overhead)
- Only comparisons, division, and math functions have overhead (~2-4 extra instructions)

## Files to Modify

1. `lightplayer/crates/lp-glsl/src/ir_utils/fixed_point.rs` - Add helper functions
2. `lightplayer/crates/lp-glsl/src/transform/fixed32/types.rs` - Update conversion
3. `lightplayer/crates/lp-glsl/src/transform/fixed32/converters/constants.rs` - Handle special constants
4. `lightplayer/crates/lp-glsl/src/transform/fixed32/converters/comparison.rs` - Implement NaN/infinity semantics
5. `lightplayer/crates/lp-glsl/src/transform/fixed32/converters/arithmetic.rs` - Update division, neg, abs
6. `lightplayer/crates/lp-glsl/src/transform/fixed32/converters/math.rs` - Update sqrt, ceil, floor
7. `lightplayer/crates/lp-glsl/tests/runtime_fixed_point.rs` - Add comprehensive tests

## Success Criteria

- All existing tests pass (backward compatibility)
- Infinity and NaN are correctly preserved in comparisons
- Division by zero produces infinity (not saturation)
- 0/0 produces NaN (not 0)
- NaN comparisons follow IEEE semantics (NaN != NaN)
- Math functions handle infinity/NaN correctly
- Basic arithmetic remains fast (no sentinel checks)
- Code size increase is minimal (~10-15% for affected operations only)




