# Phase 2: Fix SSA Dominance Violations

## Goal

Fix SSA dominance violations where variables modified in different control flow paths lack proper phi nodes, causing "uses value from non-dominating block" verifier errors.

## Problem

When variables are modified in different control flow paths (e.g., nested if statements), reading the variable after the merge point tries to use a value from a non-dominating block.

**Example Error**:
```
block1:
    v4 = iconst.i32 10
    v5 = iconst.i8 1
    brif v5, block2, block3(v6)  ; v5 = 1
;   ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
; error: inst6 (brif.i8 v5, block2, block3(v6)  ; v5 = 1): uses value arg from non-dominating block3
```

**Root Cause**: Block1 tries to branch to block3 with argument `v6`, but `v6` is defined in block3 itself, not in block1. This violates SSA dominance rules.

## Root Cause Analysis

### Issue 1: Variables Modified in Nested Scopes

When a variable is modified in an inner scope (e.g., nested if statement), and then read in an outer scope, the codegen tries to use a value that doesn't dominate the use point.

**Example**:
```glsl
int test_variable_shadowing_nested() {
    int x = 5;
    if (true) {
        int x = 10;
        if (true) {
            int x = 15;
            // Innermost x
        }
        // Middle x - tries to use innermost x, but it's out of scope
    }
    return x;  // Should use outermost x
}
```

### Issue 2: Missing Phi Nodes at Merge Points

When control flow merges (e.g., after an if statement), variables modified in different branches need phi nodes to merge their values. Cranelift's `use_var` automatically creates phi nodes, but only if called in the correct block context.

**Problem**: Variables are being read before switching to the merge block, so phi nodes aren't created correctly.

## Solution

### Solution 1: Read Variables in Correct Merge Blocks

Ensure variables are read in the merge block after all control flow paths converge. Cranelift's `use_var` automatically creates phi nodes when called in a block with multiple predecessors.

**Pattern**:
```rust
// WRONG: Read variable before branching
let x_value = ctx.builder.use_var(x_var);  // In block1
ctx.emit_cond_branch(cond, then_block, else_block)?;

// CORRECT: Read variable in merge block
ctx.emit_cond_branch(cond, then_block, else_block)?;
ctx.emit_block(then_block);
ctx.builder.def_var(x_var, new_value);
ctx.emit_branch(merge_block)?;

ctx.emit_block(else_block);
ctx.builder.def_var(x_var, other_value);
ctx.emit_branch(merge_block)?;

ctx.emit_block(merge_block);
let x_value = ctx.builder.use_var(x_var);  // Creates phi node automatically
```

### Solution 2: Proper Variable Scope Tracking

Variables declared in inner scopes must shadow outer scope variables correctly. When reading a variable, we need to find the correct variable based on scope hierarchy.

## Implementation Steps

### Step 1: Fix Variable Reading in Control Flow

1. Ensure variables are read in merge blocks, not before branching:

   - Check `lightplayer/crates/lp-glsl/src/codegen/expr/variable.rs`
   - Ensure `use_var` is called in the correct block context
   - Variables should be read after switching to merge blocks

2. Fix nested if statement handling:

   - Check `lightplayer/crates/lp-glsl/src/codegen/stmt/if.rs`
   - Ensure variables modified in then/else branches are read in merge block
   - Ensure nested if statements handle variable scope correctly

### Step 2: Fix Variable Shadowing Scope

1. Implement proper scope tracking:

   - Variables declared in inner scopes should shadow outer scope variables
   - Variable lookup should respect scope hierarchy
   - When leaving a scope, variables should go out of scope

2. Update variable declaration/lookup:

   - Check `lightplayer/crates/lp-glsl/src/codegen/stmt/declaration.rs`
   - Check `lightplayer/crates/lp-glsl/src/codegen/context.rs` - `lookup_variable` methods
   - Implement scope stack to track variable declarations

### Step 3: Test Variable Shadowing

Run variable shadowing tests:

```bash
scripts/glsl-filetests.sh control/edge_cases/variable-shadowing.glsl
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/variable.rs` - Ensure variables read in correct blocks
- `lightplayer/crates/lp-glsl/src/codegen/stmt/if.rs` - Fix variable handling in merge blocks
- `lightplayer/crates/lp-glsl/src/codegen/stmt/declaration.rs` - Implement scope tracking
- `lightplayer/crates/lp-glsl/src/codegen/context.rs` - Add scope stack for variable lookup

## Test Cases

- `control/edge_cases/variable-shadowing.glsl` - All variable shadowing tests
- `control/nested/complex.glsl` - Nested control flow with variable modifications

## Expected Behavior

- Variables modified in different control flow paths have proper phi nodes
- Variable shadowing works correctly
- No "uses value from non-dominating block" errors
- Variables are read in correct scope

## Verification

Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

Expected result: Variable shadowing tests pass, no SSA dominance violations.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix SSA dominance violations in control flow"
```

## Notes

- Cranelift's `use_var` automatically creates phi nodes when called in a block with multiple predecessors
- The key is ensuring variables are read in the correct block context
- Variable scope tracking is needed for proper shadowing behavior
- This fix depends on Phase 1 (block sealing) being correct

