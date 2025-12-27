# Phase 3: Inverse Trig Functions (asin, acos, atan, atan2)

**Goal**: Implement `asin`, `acos`, `atan`, `atan2` in `trig.glsl`.

## Tasks

1. **Implement `asin` (arc sine)**
   - Replace placeholder in `trig.glsl`
   - Use polynomial approximation or iterative method
   - Handle domain: `|x| <= 1`, return range `[-π/2, π/2]`
   - Handle edge cases: `asin(1) = π/2`, `asin(-1) = -π/2`, `asin(0) = 0`

2. **Implement `acos` (arc cosine)**
   - Replace placeholder in `trig.glsl`
   - Can use `acos(x) = π/2 - asin(x)` or dedicated algorithm
   - Handle domain: `|x| <= 1`, return range `[0, π]`
   - Handle edge cases: `acos(1) = 0`, `acos(-1) = π`, `acos(0) = π/2`

3. **Implement `atan` (arc tangent, single arg)**
   - Replace placeholder in `trig.glsl`
   - Use polynomial approximation or iterative method
   - Handle domain: all real numbers, return range `[-π/2, π/2]`
   - Handle edge cases: `atan(0) = 0`, `atan(1) = π/4`, `atan(∞) = π/2`

4. **Implement `atan2` (arc tangent, two args)**
   - Add `__lp_atan2(float y, float x)` to `trig.glsl`
   - Use `atan(y/x)` with quadrant handling
   - Handle domain: all real numbers (except both x and y are 0), return range `[-π, π]`
   - Handle edge cases: `atan2(0, 0)` undefined, `atan2(0, 1) = 0`, `atan2(1, 0) = π/2`

5. **Update loader to support `atan2`**
   - Add `atan2f` to `map_to_intrinsic_name()` if needed
   - Update `get_atan2_libcall()` to use intrinsic when `intrinsic-math` feature is enabled
   - Ensure `atan2` is in `trig` file mapping

6. **Run filetests**
   - Ensure `trig-asin.glsl`, `trig-acos.glsl`, `trig-atan.glsl` tests pass
   - Add tests for `atan2` if not already present
   - Fix any issues found

## Files to Modify

- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/trig.glsl`
- `lightplayer/crates/lp-glsl/src/frontend/intrinsics/loader.rs`
- `lightplayer/crates/lp-glsl/src/frontend/codegen/builtins/helpers.rs` (for `atan2`)

## Implementation Notes

- Can use polynomial approximations similar to CORDIC approach
- `acos` can leverage `asin` implementation: `acos(x) = π/2 - asin(x)`
- `atan2` needs quadrant detection based on signs of x and y
- Consider using existing algorithms from math libraries as reference

## Acceptance Criteria

**Test File**: `lightplayer/crates/lp-glsl-filetests/filetests/builtins/phase/03-inverse-trig.glsl`

**Run Test**:
```bash
scripts/glsl-filetests.sh builtins/phase/03-inverse-trig.glsl
```

The phase is complete when:
- [ ] `asin()`, `acos()`, `atan()`, `atan2()` work correctly from user GLSL code
- [ ] Acceptance test `03-inverse-trig.glsl` passes
- [ ] All inverse trig filetests pass
- [ ] Edge cases handled correctly (domain restrictions, special values)
- [ ] `atan2` uses intrinsic implementation when feature is enabled

