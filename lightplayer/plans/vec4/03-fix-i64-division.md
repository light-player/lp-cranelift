# Fix i64 Division in Length and Distance Functions

## Problem

The `length` and `distance` builtin functions are generating `i64 sdiv` instructions which are not supported on riscv32. The error is: "Unsupported combination: sdiv with types [types::I64] at instXX: i64 sdiv is not supported on riscv32".

**Current behavior:**
- `length(vec4)` fails with i64 sdiv error
- `distance(vec4, vec4)` fails with i64 sdiv error
- `normalize(vec4)` fails because it calls `length`

**Affected tests:**
- `vec4/builtins/distance.glsl:16` - `test_vec4_distance()` fails
- `vec4/builtins/normalize.glsl:19` - `test_vec4_normalize_length()` fails
- `vec4/edge-cases/unit-vectors.glsl:46` - `test_vec4_unit_length()` fails
- `vec4/edge-cases/zero-vector.glsl:54` - `test_vec4_zero_vector_length()` fails

## Root Cause

The `length` function computes `sqrt(dot(x, x))`. The issue is likely in:
1. How the dot product is computed (might be using i64 intermediate values)
2. How the square root is computed (might require i64 division)
3. Fixed-point arithmetic conversion issues

Looking at `lightplayer/crates/lp-glsl/src/codegen/builtins/geometric.rs`:
- `builtin_length` computes `sum_sq` by multiplying components
- Then calls `sqrt(sum_sq)`

The problem might be:
- Fixed-point conversion creating i64 values
- Square root implementation using i64 division
- Intermediate calculations overflowing to i64

## Investigation Steps

1. **Check the generated IR** for `length` function:
   - Look for where i64 sdiv is being generated
   - Check if it's in the sqrt implementation or dot product

2. **Check fixed-point conversion** (`backend/emu.rs` or similar):
   - Verify float-to-fixed conversion doesn't create i64 unnecessarily
   - Check if sqrt uses i64 division

3. **Check sqrt implementation**:
   - See how `builder.ins().sqrt()` is implemented
   - Verify it doesn't generate i64 operations

## Fix Strategy

1. **Ensure all operations stay in i32**:
   - Fixed-point arithmetic should use i32, not i64
   - Avoid operations that promote to i64

2. **Fix sqrt implementation** if needed:
   - Ensure sqrt doesn't use i64 division
   - Use i32-based sqrt algorithm if necessary

3. **Fix fixed-point conversion**:
   - Ensure float values are converted to i32 fixed-point, not i64
   - Check for any accidental i64 promotions

## Implementation Steps

1. **Add debug output** to see where i64 sdiv is generated:
   - Run a failing test and check the IR
   - Identify the exact instruction causing the issue

2. **Check sqrt implementation**:
   - Look at how `sqrt` is implemented in the backend
   - Verify it doesn't use i64 operations

3. **Fix the root cause**:
   - If sqrt uses i64, implement i32-based sqrt
   - If conversion creates i64, fix the conversion
   - If intermediate calculations overflow, use different approach

4. **Test with simple cases**:
   - `length(vec4(1.0, 0.0, 0.0, 0.0))` should return 1.0
   - `length(vec4(0.0, 0.0, 0.0, 0.0))` should return 0.0

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/builtins/geometric.rs` - `builtin_length` and `builtin_distance`
- Possibly `lightplayer/crates/lp-glsl/src/backend/emu.rs` - sqrt implementation
- Possibly fixed-point conversion code

## Test Cases

- `vec4/builtins/distance.glsl:16` - `test_vec4_distance()` should pass
- `vec4/builtins/normalize.glsl:19` - `test_vec4_normalize_length()` should pass
- `vec4/edge-cases/unit-vectors.glsl:46` - `test_vec4_unit_length()` should pass
- `vec4/edge-cases/zero-vector.glsl:54` - `test_vec4_zero_vector_length()` should pass

## Acceptance Criteria

- [ ] All `length` function tests pass
- [ ] All `distance` function tests pass
- [ ] All `normalize` function tests pass (depends on length)
- [ ] No i64 sdiv errors in any vec4 tests
- [ ] Code compiles without warnings

## Verification

Run the failing tests:
```bash
scripts/glsl-filetests.sh vec4/builtins/distance.glsl:16
scripts/glsl-filetests.sh vec4/builtins/normalize.glsl:19
scripts/glsl-filetests.sh vec4/edge-cases/unit-vectors.glsl:46
scripts/glsl-filetests.sh vec4/edge-cases/zero-vector.glsl:54
```

Expected result: Tests pass with no i64 sdiv errors.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix i64 division in length and distance functions"
```



