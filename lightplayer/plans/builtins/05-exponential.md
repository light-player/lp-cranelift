# Phase 5: Exponential Functions (exp, log, exp2, log2, pow)

**Goal**: Create `exponential.glsl` and implement `exp`, `log`, `exp2`, `log2`, `pow`.

## Tasks

1. **Create `exponential.glsl` file**
   - Create new file in `frontend/intrinsics/`
   - Add placeholder implementations for all exponential functions
   - Ensure file is self-contained (no cross-file dependencies)

2. **Update loader to support exponential functions**
   - Add mappings in `map_to_intrinsic_name()`: `expf`, `logf`, `exp2f`, `log2f`, `powf`
   - Add `exponential` to `get_intrinsic_file()` mapping
   - Update `get_intrinsic_file()` to return `"exponential"` for these functions

3. **Implement `exp` (natural exponential)**
   - Implement `__lp_exp(float x)` in `exponential.glsl`
   - Use polynomial approximation or iterative method
   - Handle edge cases: `exp(0) = 1`, `exp(1) = e`, large/small values, overflow

4. **Implement `log` (natural logarithm)**
   - Implement `__lp_log(float x)` in `exponential.glsl`
   - Use polynomial approximation or iterative method
   - Handle domain: `x > 0`, undefined for `x <= 0`
   - Handle edge cases: `log(1) = 0`, `log(e) = 1`

5. **Implement `exp2` (base-2 exponential)**
   - Implement `__lp_exp2(float x)` in `exponential.glsl`
   - Can use `exp2(x) = exp(x * ln(2))` or dedicated algorithm
   - Handle edge cases: `exp2(0) = 1`, `exp2(1) = 2`

6. **Implement `log2` (base-2 logarithm)**
   - Implement `__lp_log2(float x)` in `exponential.glsl`
   - Can use `log2(x) = log(x) / log(2)` or dedicated algorithm
   - Handle domain: `x > 0`
   - Handle edge cases: `log2(1) = 0`, `log2(2) = 1`

7. **Implement `pow` (power function)**
   - Implement `__lp_pow(float x, float y)` in `exponential.glsl`
   - Use `pow(x, y) = exp(log(x) * y)` for most cases
   - Add special handling for integer powers (optimization)
   - Handle edge cases: `pow(0, 0)` undefined, `pow(x, 0) = 1`, `pow(1, y) = 1`

8. **Update codegen to use intrinsics**
   - Ensure `builtin_exp()`, `builtin_log()`, `builtin_exp2()`, `builtin_log2()`, `builtin_pow()` call `get_math_libcall()`
   - Verify they map to correct intrinsic names

9. **Run filetests**
   - Ensure `exp-exp.glsl`, `exp-log.glsl`, `exp-exp2.glsl`, `exp-log2.glsl`, `exp-pow.glsl` tests pass
   - Fix any issues found

## Files to Create/Modify

- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/exponential.glsl` (new file)
- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/loader.rs`
- `lightplayer/crates/lp-glsl/src/frontend/codegen/builtins/` (verify exponential functions use intrinsics)

## Implementation Notes

- Use polynomial approximations similar to CORDIC for trig functions
- Can reference algorithms from math libraries (e.g., Remez algorithm for polynomials)
- Target ±2-3 ULP accuracy initially
- `pow` implementation can be optimized later with dedicated algorithm

## Acceptance Criteria

**Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/05-exponential.glsl`

**Run Test**:
```bash
scripts/glsl-filetests.sh builtins/phase/05-exponential.glsl
```

The phase is complete when:
- [ ] `exponential.glsl` file created and self-contained
- [ ] All exponential functions work correctly from user GLSL code
- [ ] Acceptance test `05-exponential.glsl` passes
- [ ] All exponential filetests pass
- [ ] Edge cases handled correctly (domain restrictions, overflow, special values)
- [ ] Functions are properly mapped in loader

