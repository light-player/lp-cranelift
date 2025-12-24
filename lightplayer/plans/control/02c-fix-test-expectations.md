# Phase 2c: Fix Test Expectations for Variable Shadowing

## Goal

Correct test expectations that have incorrect expected values due to misunderstanding of variable shadowing semantics.

## Problem

Several test files have incorrect expected values. After implementing proper scope management in Phase 2, some tests are failing because the expected values don't match the correct C/GLSL scoping semantics:

1. **`control/if/variable-scope.glsl:41`**: `test_if_variable_shadowing` expects `10` but should be `5` (outer x unchanged when inner x shadows it)
2. **`control/loop_for/variable-scope.glsl:33`**: `test_for_loop_init_shadowing` expects `3` but should be `100` (outer i unchanged when loop i shadows it)
3. **`control/loop_while/variable-scope.glsl:38`**: `test_while_loop_shadowing` expects `20` but should be `100` (outer i unchanged when loop body i shadows it)

## Root Cause

These tests were written with incorrect expectations. When a variable is shadowed in an inner scope, the outer variable should remain unchanged. The tests were expecting the shadowed value instead of the outer value.

## Solution

Update the test expectations to match correct C/GLSL scoping semantics:

1. **`test_if_variable_shadowing`**: Change expected value from `10` to `5`
   - Outer `x = 5`, inner `x = 10` shadows it, but outer `x` remains `5`

2. **`test_for_loop_init_shadowing`**: Change expected value from `3` to `100`
   - Outer `i = 100`, loop `i` shadows it, but outer `i` remains `100`

3. **`test_while_loop_shadowing`**: Change expected value from `20` to `100`
   - Outer `i = 100`, loop body `i` shadows it, but outer `i` remains `100`

## Implementation Steps

1. **Review each failing test** to understand what it's actually testing
2. **Verify the correct expected value** based on C/GLSL scoping rules
3. **Update test expectations** in the test files
4. **Run tests** to verify they now pass with correct expectations

## Files to Modify

- `lightplayer/crates/lp-glsl-filetests/filetests/control/if/variable-scope.glsl` - Fix `test_if_variable_shadowing` expectation
- `lightplayer/crates/lp-glsl-filetests/filetests/control/loop_for/variable-scope.glsl` - Fix `test_for_loop_init_shadowing` expectation
- `lightplayer/crates/lp-glsl-filetests/filetests/control/loop_while/variable-scope.glsl` - Fix `test_while_loop_shadowing` expectation

## Test Cases to Verify

After fixing expectations, verify these tests pass:

```bash
scripts/glsl-filetests.sh control/if/variable-scope.glsl
scripts/glsl-filetests.sh control/loop_for/variable-scope.glsl
scripts/glsl-filetests.sh control/loop_while/variable-scope.glsl
```

## Expected Behavior

- Tests reflect correct C/GLSL scoping semantics
- Outer variables remain unchanged when inner variables shadow them
- All variable scope tests pass with corrected expectations

## Verification

Run all variable scope tests:

```bash
scripts/glsl-filetests.sh control/if/variable-scope.glsl
scripts/glsl-filetests.sh control/loop_for/variable-scope.glsl
scripts/glsl-filetests.sh control/loop_while/variable-scope.glsl
scripts/glsl-filetests.sh control/loop_do_while/variable-scope.glsl
```

Expected result: All tests pass with correct expectations.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix test expectations for variable shadowing"
```

## Notes

- This should be done after Phase 2 (SSA dominance) and Phase 2a/2b (do-while fixes)
- These are test fixes, not code fixes - the implementation is correct
- Verify each test case manually to ensure the expected value is correct

