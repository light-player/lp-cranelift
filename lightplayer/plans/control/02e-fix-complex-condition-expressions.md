# Phase 2e: Fix Complex Condition Expressions

## Goal

Ensure complex condition expressions (using `&&`, `||`, comparisons) work correctly in all control flow contexts (if, while, for, do-while).

## Problem

Tests with complex condition expressions are failing:
- `control/loop_for/complex-condition.glsl` - Complex conditions in for loops
- `control/loop_while/complex-condition.glsl` - Complex conditions in while loops
- `control/edge_cases/condition-expressions.glsl` - General condition expression tests

These failures could be due to:
1. Logical OR operator (`||`) not implemented (Phase 6)
2. Logical AND operator (`&&`) not working correctly
3. Condition evaluation not handling complex expressions
4. Type conversion issues in condition expressions

## Current Implementation

- While loops use `translate_condition()` which validates boolean type
- For loops use `translate_condition()` for conditions
- Do-while loops (after Phase 2b) should use proper type handling
- If statements validate condition is boolean type

## Potential Issues

1. **Logical OR (`||`)**: May not be implemented (Phase 6)
2. **Logical AND (`&&`)**: May not be working correctly
3. **Short-circuit evaluation**: May not be implemented for `&&` and `||`
4. **Condition type handling**: May not handle complex expressions correctly

## Solution

1. **Check if logical OR is implemented**:
   - Search for `||` operator implementation
   - If missing, this is Phase 6 (separate feature)
   - Tests using `||` will fail until Phase 6 is complete

2. **Verify logical AND works correctly**:
   - Test `&&` operator in conditions
   - Ensure it returns boolean type
   - Verify short-circuit evaluation if implemented

3. **Fix condition expression handling**:
   - Ensure complex expressions are evaluated correctly
   - Ensure type validation works for complex expressions
   - Ensure boolean conversion works correctly

4. **Handle short-circuit evaluation** (if needed):
   - `&&` should short-circuit if left side is false
   - `||` should short-circuit if left side is true
   - This may require control flow changes

## Implementation Steps

1. **Check logical operator implementation**:
   ```bash
   grep -r "logical_or\|LogicalOr\|||" lightplayer/crates/lp-glsl/src/codegen/
   grep -r "logical_and\|LogicalAnd\|&&" lightplayer/crates/lp-glsl/src/codegen/
   ```

2. **Test condition expressions**:
   ```bash
   scripts/glsl-filetests.sh control/edge_cases/condition-expressions.glsl
   scripts/glsl-filetests.sh control/loop_for/complex-condition.glsl
   scripts/glsl-filetests.sh control/loop_while/complex-condition.glsl
   ```

3. **Identify root cause**:
   - If `||` is missing, note that Phase 6 is needed
   - If `&&` is broken, fix it
   - If condition evaluation is broken, fix it

4. **Fix identified issues** (if not Phase 6):
   - Implement missing operators
   - Fix condition evaluation
   - Fix type handling

## Files to Review/Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Check logical operator implementations
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loops.rs` - Check `translate_condition()` implementation
- `lightplayer/crates/lp-glsl/src/codegen/stmt/if.rs` - Check condition handling
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Check condition handling (after Phase 2b)

## Test Cases

- `control/edge_cases/condition-expressions.glsl` - General condition tests
- `control/loop_for/complex-condition.glsl` - Complex conditions in for loops
- `control/loop_while/complex-condition.glsl` - Complex conditions in while loops
- Tests using `||` will fail until Phase 6 is complete

## Expected Behavior

- Logical AND (`&&`) works correctly in conditions
- Logical OR (`||`) works correctly in conditions (after Phase 6)
- Complex condition expressions evaluate correctly
- Type validation works for complex expressions
- All condition expression tests pass (except those requiring Phase 6)

## Verification

Run condition expression tests:

```bash
scripts/glsl-filetests.sh control/edge_cases/condition-expressions.glsl
scripts/glsl-filetests.sh control/loop_for/complex-condition.glsl
scripts/glsl-filetests.sh control/loop_while/complex-condition.glsl
```

Expected result: Tests pass, except those requiring logical OR (Phase 6).

## Commit Instructions

Once fixes are complete:

```bash
git add -A
git commit -m "lpc: fix complex condition expression handling"
```

## Notes

- This should be done after Phase 2b (do-while condition handling)
- Logical OR (`||`) is Phase 6 - separate feature
- Some tests may fail until Phase 6 is complete
- Focus on fixing `&&` and condition evaluation, not `||`

