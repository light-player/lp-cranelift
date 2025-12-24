# Phase 3: Fix Loop Variable Preservation

## Goal

Fix loop variable preservation so loop variables are correctly preserved after loop exit, especially when modified in the loop body.

## Problem

Loop variables are not correctly preserved after loop exit. In particular, for loops with update expressions execute the update even when the loop variable is modified in the body, causing incorrect values.

**Example**:
```glsl
int test_for_loop_expression_modified_in_body() {
    int i = 0;
    for (i = 0; i < 5; i++) {
        i = i + 1; // Modify loop variable in body
    }
    return i;
    // Expected: 5 (value after loop exits)
    // Actual: 6 (double increment: once in body, once in update)
}
```

**Error**: `run test failed at line 30: expected 5, got 6`

## Root Cause Analysis

### Issue 1: For Loop Update Expression Execution

In for loops, the update expression (`i++` in `for (i = 0; i < 5; i++)`) is executed after each loop iteration, even when the loop variable is modified in the body.

**Current Behavior**:
1. Loop iteration starts with `i = 0`
2. Body executes: `i = i + 1` → `i = 1`
3. Update expression executes: `i++` → `i = 2`
4. Next iteration: `i = 2`
5. Body executes: `i = i + 1` → `i = 3`
6. Update expression executes: `i++` → `i = 4`
7. ...continues until `i >= 5`
8. Final value: `i = 6` (should be 5)

**Problem**: The update expression should use the value from the body, not the value before the body. But the current implementation reads the variable before the body executes.

### Issue 2: Loop Variable Not Preserved After Break

When a loop exits via `break`, the loop variable should preserve its value at the break point, but it may not be correctly preserved.

**Example**:
```glsl
int test_for_loop_expression_break() {
    int i = 0;
    for (i = 0; i < 10; i++) {
        if (i >= 5) {
            break;
        }
    }
    return i;
    // Expected: 5 (value when break occurred)
}
```

## Solution

### Solution 1: Fix For Loop Update Expression

The update expression should read the loop variable value from after the body executes, not before. This requires proper block ordering and variable reading.

**Correct Pattern**:
```
init_block:
    i = 0
    jump header_block

header_block(i: i32):
    if i < 5:
        jump body_block
    else:
        jump exit_block

body_block:
    i = i + 1  // Modify in body
    jump update_block

update_block:
    i = i + 1  // Update expression uses value from body_block
    jump header_block(i)  // Pass updated value back to header

exit_block:
    return i  // Should be 5
```

**Key Insight**: The update block should read the variable value from the body block, not from before the body. This requires proper phi node handling.

### Solution 2: Ensure Loop Variable Preservation

When a loop exits (via break or condition), the loop variable should be read in the exit block, ensuring proper phi node creation.

## Implementation Steps

### Step 1: Fix For Loop Update Expression

1. Update `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_for.rs`:

   - Ensure update expression reads variable value from after body executes
   - The update block should read the variable in its own block context
   - Ensure proper phi node creation for loop variable

2. Key changes:

   ```rust
   // Update block
   ctx.emit_block(update_block);
   if let Some(update_expr) = &rest.post_expr {
       // Read loop variable in update block context (creates phi node)
       // This ensures we get the value from after body execution
       ctx.translate_expr(update_expr)?;
   }
   ctx.emit_branch(header_block)?;
   ```

### Step 2: Fix Loop Variable Reading After Exit

1. Ensure loop variables are read in exit blocks:

   - When reading loop variable after loop exit, read it in the exit block
   - This ensures proper phi node creation if variable was modified in loop

2. Test with break statements:

   ```bash
   scripts/glsl-filetests.sh control/edge_cases/loop-expression-scope.glsl:32
   ```

### Step 3: Test Loop Variable Preservation

Run loop expression scope tests:

```bash
scripts/glsl-filetests.sh control/edge_cases/loop-expression-scope.glsl
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_for.rs` - Fix update expression handling
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_while.rs` - Ensure variable preservation
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Ensure variable preservation

## Test Cases

- `control/edge_cases/loop-expression-scope.glsl` - All loop variable preservation tests
  - `test_for_loop_expression_preserved()` - Basic preservation
  - `test_for_loop_expression_modified_in_body()` - Modified in body
  - `test_for_loop_expression_break()` - Break handling
  - `test_for_loop_expression_continue()` - Continue handling
  - `test_while_loop_variable_preserved()` - While loop preservation
  - `test_do_while_loop_variable_preserved()` - Do-while loop preservation

## Expected Behavior

- Loop variables are correctly preserved after loop exit
- Update expressions use correct variable values
- Break statements preserve loop variable value correctly
- Continue statements preserve loop variable value correctly

## Verification

Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

Expected result: Loop expression scope tests pass, loop variables preserved correctly.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix loop variable preservation after loop exit"
```

## Notes

- This fix depends on Phase 2 (SSA dominance) being correct - proper phi nodes are needed
- The key insight is that update expressions should read variables in their own block context
- Cranelift's `use_var` automatically creates phi nodes when reading in blocks with multiple predecessors
- Loop variables modified in body need proper phi nodes to merge values from body and update blocks

