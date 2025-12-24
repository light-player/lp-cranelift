# Phase 2a: Fix Do-While Loop Scope Management

## Goal

Add proper scope management to do-while loops so variables declared in loop bodies are properly scoped and don't leak to outer scopes.

## Problem

Do-while loops are missing `enter_scope()` and `exit_scope()` calls that are present in for and while loops. This causes:

- Variables declared in do-while bodies not being properly scoped
- Variable shadowing not working correctly in do-while loops
- Potential variable leakage to outer scopes

## Current Code

```rust
// Body: switch to but don't seal yet - will receive back edge from condition block
ctx.switch_to_block(body_block);
ctx.emit_statement(body)?;  // Missing scope management!
ctx.emit_branch(cond_block)?;
```

## Solution

Add scope management around the body statement, matching the pattern used in while loops:

```rust
// Body: switch to but don't seal yet - will receive back edge from condition block
ctx.switch_to_block(body_block);
ctx.enter_scope(); // Enter scope for body variables
ctx.emit_statement(body)?;
ctx.exit_scope(); // Exit scope for body variables
ctx.emit_branch(cond_block)?;
```

## Implementation Steps

1. **Update `emit_loop_do_while_stmt` in `loop_do_while.rs`**:

   - Add `ctx.enter_scope()` before emitting the body statement
   - Add `ctx.exit_scope()` after emitting the body statement
   - Ensure scope is exited before branching to condition block

2. **Test with do-while variable scope tests**:
   ```bash
   scripts/glsl-filetests.sh control/loop_do_while/variable-scope.glsl
   ```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Add scope management

## Test Cases

- `control/loop_do_while/variable-scope.glsl` - All do-while variable scope tests
- `control/loop_do_while/basic.glsl` - Basic do-while tests (should still pass)
- `control/loop_do_while/nested.glsl` - Nested do-while tests

## Expected Behavior

- Variables declared in do-while bodies are scoped to the loop body
- Variables declared in do-while bodies don't leak to outer scopes
- Variable shadowing works correctly in do-while loops
- All existing do-while tests continue to pass

## Verification

Run all do-while tests:

```bash
scripts/glsl-filetests.sh control/loop_do_while/
```

Expected result: Variable scope tests pass, no regressions in basic tests.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: add scope management to do-while loops"
```

## Notes

- This is a simple fix that matches the pattern already used in while loops
- Should be done before Phase 2b (condition type handling) as it's a prerequisite
- This fix is independent of block sealing issues (Phase 1)
