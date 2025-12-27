# Fix Verifier Errors in Vector Indexing

## Problem

Vector array indexing in conditional expressions causes verifier errors: "uses value from non-dominating inst". This happens when comparing vector components accessed via array indexing inside `if` statements.

**Current behavior:**

- Verifier error: "uses value v13 from non-dominating inst13"
- Compilation fails with verifier errors

**Affected tests:**

- `vec4/indexing/array-indexing.glsl:43` - `test_vec4_array_indexing_equals_component()` fails
- `vec4/indexing/component-access.glsl:58` - `test_vec4_component_access_verify_synonyms()` fails

## Root Cause

The verifier error shows that values computed in one branch of an `if` statement are being used in subsequent code that doesn't dominate them. This is a codegen issue where:

1. Values are computed inside `if` blocks (e.g., `sum = sum + 1.0`)
2. These values are then used in comparisons outside the block
3. The SSA form is invalid because the value doesn't dominate its use

Looking at the error:

```
block2(v11: i32):
    v14 = iconst.i32 1
    v15 = icmp.i32 eq v1, v13  ; v1 = 0x0002_0000
;   ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
; error: inst16 (v15 = icmp.i32 eq v1, v13  ; v1 = 0x0002_0000): uses value v13 from non-dominating inst13
```

`v13` is computed in `block1` but used in `block2` which doesn't dominate it.

## Investigation Steps

1. **Check codegen for conditional expressions** (`codegen/expr/mod.rs`):

   - Look at how `if` statements are translated
   - Verify phi nodes are created correctly for values modified in branches

2. **Check vector indexing codegen** (`codegen/expr/component.rs`):

   - Verify array indexing doesn't create values that escape their scope incorrectly

3. **Check SSA construction**:
   - Ensure all values used across basic blocks are properly phi'd
   - Verify the control flow graph is correct

## Fix Strategy

The issue is likely in how conditional expressions handle values that are:

1. Read from vectors (via indexing)
2. Used in comparisons
3. Modified in branches

The fix should ensure:

1. Values read from vectors are properly available in all branches
2. Values modified in branches are properly phi'd at merge points
3. The SSA form is valid

## Implementation Steps

1. **Trace through the failing test** to understand the exact codegen flow:

   - `test_vec4_array_indexing_equals_component()` compares `v[0] == v.x`
   - This happens inside an `if` statement that modifies `sum`

2. **Check conditional expression translation**:

   - Verify that values read before the `if` are available in all branches
   - Verify that values modified in branches are phi'd correctly

3. **Fix the codegen**:

   - Ensure vector component reads happen before branching
   - Ensure all modified values are properly phi'd at merge points

4. **Test with simpler cases first**:
   - Start with `if (v[0] == v.x) sum = sum + 1.0;`
   - Then test nested conditionals

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Conditional expression translation
- `lightplayer/crates/lp-glsl/src/codegen/expr/component.rs` - Vector indexing codegen
- Possibly `lightplayer/crates/lp-glsl/src/codegen/context.rs` - Value management

## Test Cases

- `vec4/indexing/array-indexing.glsl:43` - `test_vec4_array_indexing_equals_component()` should pass
- `vec4/indexing/component-access.glsl:58` - `test_vec4_component_access_verify_synonyms()` should pass
- Verify no regressions in other tests

## Acceptance Criteria

- [ ] `test_vec4_array_indexing_equals_component()` passes
- [ ] `test_vec4_component_access_verify_synonyms()` passes
- [ ] All tests in `vec4/indexing/` pass
- [ ] No verifier errors in any vec4 tests
- [ ] Code compiles without warnings

## Verification

Run the failing tests:

```bash
scripts/glsl-filetests.sh vec4/indexing/array-indexing.glsl:43
scripts/glsl-filetests.sh vec4/indexing/component-access.glsl:58
```

Expected result: Tests pass with no verifier errors.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix verifier errors in vector indexing conditionals"
```

