# Implement Vector Comparison Operators

## Problem

Vector comparison operators `==` and `!=` are not supported on vectors. The error is: "operator Equal not supported on vectors yet" and "operator NonEqual not supported on vectors yet".

**Current behavior:**
- `vec4 == vec4` fails with compilation error
- `vec4 != vec4` fails with compilation error

**Affected tests:**
- `vec4/relational/equal.glsl:16` - `test_vec4_equal_operator()` fails
- `vec4/relational/not-equal.glsl:16` - `test_vec4_not_equal_operator()` fails

## Root Cause

In `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs`, the operator translation for `Equal` and `NonEqual` likely only handles scalars and doesn't support vectors.

GLSL spec says:
- `vec4 == vec4` → `bool` (aggregate comparison: true if all components equal)
- `vec4 != vec4` → `bool` (aggregate comparison: true if any component differs)

## Fix Strategy

1. **Update operator translation** in `codegen/expr/mod.rs`:
   - Detect when operands are vectors
   - For `==`: Compare all components, AND results together
   - For `!=`: Compare all components, OR results together (or use NOT of ==)

2. **Handle component-wise comparison**:
   - Compare each component pair
   - Combine results appropriately

## Implementation Steps

1. **Find operator translation code** in `codegen/expr/mod.rs`:
   - Look for `translate_binary_op` or similar
   - Find where `Equal` and `NonEqual` are handled

2. **Add vector support**:
   - Check if both operands are vectors
   - If so, perform component-wise comparison
   - Combine results:
     - `==`: AND all component comparisons
     - `!=`: OR all component comparisons (or NOT of ==)

3. **Test with different cases**:
   - Equal vectors: `vec4(1,2,3,4) == vec4(1,2,3,4)` → true
   - Different vectors: `vec4(1,2,3,4) == vec4(1,2,3,5)` → false
   - Partially equal: `vec4(1,2,3,4) != vec4(1,2,3,5)` → true

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Operator translation
- Possibly `lightplayer/crates/lp-glsl/src/semantic/type_check/` - Type checking

## Test Cases

- `vec4/relational/equal.glsl:16` - `test_vec4_equal_operator()` should pass
- `vec4/relational/not-equal.glsl:16` - `test_vec4_not_equal_operator()` should pass
- Verify works for vec2, vec3, vec4
- Verify works for ivec2, ivec3, ivec4

## Acceptance Criteria

- [ ] `test_vec4_equal_operator()` passes
- [ ] `test_vec4_not_equal_operator()` passes
- [ ] All tests in `vec4/relational/equal.glsl` pass
- [ ] All tests in `vec4/relational/not-equal.glsl` pass
- [ ] No regressions in other tests
- [ ] Code compiles without warnings

## Verification

Run the tests:
```bash
scripts/glsl-filetests.sh vec4/relational/equal.glsl:16
scripts/glsl-filetests.sh vec4/relational/not-equal.glsl:16
```

Expected result: Tests pass, `==` and `!=` work correctly on vectors.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement vector comparison operators == and !="
```

