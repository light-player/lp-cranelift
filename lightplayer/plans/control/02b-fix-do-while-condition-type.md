# Phase 2b: Fix Do-While Loop Condition Type Handling

## Goal

Ensure do-while loop conditions are properly converted to boolean (i8) type before branching, matching the behavior of while loops.

## Problem

Do-while loops use `translate_expr()` directly, which may return `i32` instead of `i8` (bool). The condition needs to be properly validated and converted to boolean type before use in `emit_cond_branch()`.

**Current code**:

```rust
let condition_value = ctx.translate_expr(condition)?;  // Might return i32!
ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
```

**Issue**: While loops use `translate_condition()` which validates and ensures boolean type, but do-while doesn't.

## Solution

Use the same `translate_condition()` helper that while loops use, or ensure proper boolean conversion. However, since do-while uses `Expr` not `Condition`, we need to:

1. Translate the expression and validate it's boolean type
2. Extract the boolean value (first component for scalar bool)
3. Use that value for branching

**Pattern** (matching while loop approach):

```rust
let (cond_vals, cond_ty) = ctx.translate_expr_typed(condition)?;
// Validate that condition is bool type (GLSL spec requirement)
if cond_ty != crate::semantic::types::Type::Bool {
    // Error handling
}
// Condition must be scalar, so we take the first (and only) value
let condition_value = cond_vals.into_iter().next().ok_or_else(|| {
    GlslError::new(ErrorCode::E0400, "condition expression produced no value")
})?;
ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
```

## Implementation Steps

1. **Update `emit_loop_do_while_stmt` in `loop_do_while.rs`**:

   - Replace `ctx.translate_expr(condition)?` with `ctx.translate_expr_typed(condition)?`
   - Add type validation to ensure condition is `Type::Bool`
   - Extract the first (and only) value from the condition values
   - Add proper error handling for invalid condition types

2. **Test with complex condition tests**:
   ```bash
   scripts/glsl-filetests.sh control/loop_do_while/basic.glsl
   scripts/glsl-filetests.sh control/edge_cases/condition-expressions.glsl
   ```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Fix condition type handling

## Test Cases

- `control/loop_do_while/basic.glsl` - All basic do-while tests
- `control/edge_cases/condition-expressions.glsl` - Complex condition expressions
- `control/loop_do_while/complex-condition.glsl` - If such tests exist

## Expected Behavior

- Do-while conditions are properly validated as boolean type
- Type errors are reported for non-boolean conditions
- Complex condition expressions work correctly
- All existing do-while tests continue to pass

## Verification

Run all do-while and condition expression tests:

```bash
scripts/glsl-filetests.sh control/loop_do_while/
scripts/glsl-filetests.sh control/edge_cases/condition-expressions.glsl
```

Expected result: All tests pass, proper type validation.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix do-while condition type handling"
```

## Notes

- This should be done after Phase 2a (scope management)
- Matches the pattern used in while loops for consistency
- Ensures GLSL spec compliance (conditions must be bool type)
