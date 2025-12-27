# Phase 4: Hyperbolic Trig Functions (sinh, cosh, tanh, asinh, acosh, atanh)

**Goal**: Implement `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh` in `trig.glsl`.

## Tasks

1. **Implement `sinh` (hyperbolic sine)**
   - Replace placeholder in `trig.glsl`
   - Use formula: `sinh(x) = (e^x - e^(-x)) / 2`
   - Can use `exp()` function (will need to handle this carefully since `exp` isn't implemented yet)
   - Alternative: Use polynomial approximation directly
   - Handle edge cases: `sinh(0) = 0`, large values

2. **Implement `cosh` (hyperbolic cosine)**
   - Replace placeholder in `trig.glsl`
   - Use formula: `cosh(x) = (e^x + e^(-x)) / 2`
   - Can use `exp()` or polynomial approximation
   - Handle edge cases: `cosh(0) = 1`, large values

3. **Implement `tanh` (hyperbolic tangent)**
   - Replace placeholder in `trig.glsl`
   - Use formula: `tanh(x) = sinh(x) / cosh(x)`
   - Can call `sinh` and `cosh` implementations
   - Handle edge cases: `tanh(0) = 0`, `tanh(∞) = 1`, `tanh(-∞) = -1`

4. **Implement `asinh` (inverse hyperbolic sine)**
   - Replace placeholder in `trig.glsl`
   - Use formula: `asinh(x) = ln(x + sqrt(x^2 + 1))`
   - Will need `log` and `sqrt` (may need to implement basic versions or wait for exponential phase)
   - Alternative: Use polynomial approximation
   - Handle edge cases: `asinh(0) = 0`

5. **Implement `acosh` (inverse hyperbolic cosine)**
   - Replace placeholder in `trig.glsl`
   - Use formula: `acosh(x) = ln(x + sqrt(x^2 - 1))` for `x >= 1`
   - Domain: `x >= 1`, undefined for `x < 1`
   - Handle edge cases: `acosh(1) = 0`

6. **Implement `atanh` (inverse hyperbolic tangent)**
   - Replace placeholder in `trig.glsl`
   - Use formula: `atanh(x) = 0.5 * ln((1 + x) / (1 - x))` for `|x| < 1`
   - Domain: `|x| < 1`, undefined for `|x| >= 1`
   - Handle edge cases: `atanh(0) = 0`

7. **Run filetests**
   - Ensure `trig-sinh.glsl`, `trig-cosh.glsl`, `trig-tanh.glsl` tests pass
   - Ensure `trig-asinh.glsl`, `trig-acosh.glsl`, `trig-atanh.glsl` tests pass
   - Fix any issues found

## Files to Modify

- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/trig.glsl`

## Implementation Notes

- **Decision needed**: Should hyperbolic functions use `exp()`, `log()`, `sqrt()` or have their own approximations?
  - Option A: Use polynomial approximations directly (no dependencies)
  - Option B: Implement basic versions of `exp`, `log`, `sqrt` inline (self-contained)
  - Option C: Wait for exponential phase, then update hyperbolic functions
  
- **Recommendation**: Option B - implement basic inline versions for now, can refactor later

- Hyperbolic functions can use Taylor series or polynomial approximations
- Inverse hyperbolic functions can use logarithmic formulas or approximations

## Acceptance Criteria

**Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/04-hyperbolic-trig.glsl`

**Run Test**:
```bash
scripts/glsl-filetests.sh builtins/phase/04-hyperbolic-trig.glsl
```

The phase is complete when:
- [ ] All hyperbolic trig functions work correctly from user GLSL code
- [ ] Acceptance test `04-hyperbolic-trig.glsl` passes
- [ ] All hyperbolic trig filetests pass
- [ ] Edge cases handled correctly (domain restrictions, special values)
- [ ] Functions are self-contained (no external dependencies if using Option B)

