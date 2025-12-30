# Implement Vector Shortening Constructors

## Problem

Vector shortening constructors like `vec2(vec4)` are not supported. The error is: "cannot construct `vec2` from `Vec4` - expected 2 components, found 4".

**Current behavior:**
- `vec2 v2 = vec2(v4);` fails with compilation error
- Expected: Should drop z and w components, keeping x and y

**Affected tests:**
- `vec4/constructors/shortening.glsl:16` - `test_vec2_from_vec4()` fails

## Root Cause

In `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs`, the constructor logic likely checks that the number of components matches exactly. It should allow constructing smaller vectors from larger ones by dropping trailing components.

GLSL spec allows:
- `vec2(vec4)` - takes first 2 components (x, y)
- `vec3(vec4)` - takes first 3 components (x, y, z)
- `vec2(vec3)` - takes first 2 components (x, y)
- etc.

## Fix Strategy

1. **Update constructor type checking** to allow shortening:
   - When constructing a smaller vector from a larger one, allow it
   - Extract only the needed components (first N components)

2. **Update constructor codegen** to extract components:
   - For `vec2(vec4)`, extract components [0, 1] (x, y)
   - For `vec3(vec4)`, extract components [0, 1, 2] (x, y, z)
   - Drop the remaining components

3. **Support all vector types**:
   - `vec2`, `vec3`, `vec4` shortening
   - `ivec2`, `ivec3`, `ivec4` shortening
   - `bvec2`, `bvec3`, `bvec4` shortening

## Implementation Steps

1. **Find constructor type checking code** (`codegen/expr/constructor.rs` or `semantic/type_check/`):
   - Look for where it checks component count
   - Update to allow source components >= target components

2. **Update constructor codegen**:
   - When source has more components than target, extract first N components
   - Use component access to get the needed values

3. **Test with different combinations**:
   - `vec2(vec4)` - should work
   - `vec3(vec4)` - should work
   - `vec2(vec3)` - should work
   - Verify component order is preserved (x, y, z, w → x, y)

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs` - Constructor codegen
- Possibly `lightplayer/crates/lp-glsl/src/semantic/type_check/inference.rs` - Type checking

## Test Cases

- `vec4/constructors/shortening.glsl:16` - `test_vec2_from_vec4()` should pass
- Verify `vec3(vec4)` also works
- Verify `vec2(vec3)` works
- Verify component order is correct (x, y from vec4)

## Acceptance Criteria

- [ ] `test_vec2_from_vec4()` passes
- [ ] All tests in `vec4/constructors/shortening.glsl` pass
- [ ] No regressions in other constructor tests
- [ ] Code compiles without warnings

## Verification

Run the test:
```bash
scripts/glsl-filetests.sh vec4/constructors/shortening.glsl:16
```

Expected result: Test passes, `vec2(v4)` correctly extracts x and y components.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement vector shortening constructors"
```




