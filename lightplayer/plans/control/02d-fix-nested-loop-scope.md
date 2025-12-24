# Phase 2d: Fix Nested Loop Scope Management

## Goal

Ensure nested loops properly manage variable scopes so inner loop variables shadow outer loop variables correctly, and variables are properly scoped at each nesting level.

## Problem

Nested loops may have scope management issues:
- Inner loop variables not properly shadowing outer loop variables
- Variables from outer loops leaking into inner loops incorrectly
- Scope not being properly entered/exited for nested loop bodies
- Variables declared in loop bodies not being scoped correctly

## Current Implementation

For loops enter scope before init, exit after loop:
```rust
ctx.enter_scope();  // Enter scope for loop variables
// ... init, header, body, update ...
ctx.exit_scope();   // Exit scope for loop variables
```

While loops enter scope for body only:
```rust
ctx.enter_scope(); // Enter scope for body variables
ctx.emit_statement(body)?;
ctx.exit_scope(); // Exit scope for body variables
```

Do-while loops (after Phase 2a) should match while loops.

## Potential Issues

1. **Nested for loops**: Both loops enter scope, but inner loop scope might not properly shadow outer loop scope
2. **Nested while loops**: Body scopes might interfere with each other
3. **Mixed nesting**: For loop containing while loop, or vice versa
4. **Variable lookup**: Scope stack search might not be working correctly for nested scopes

## Solution

The scope stack implementation should already handle this correctly (innermost to outermost search), but we need to verify:

1. **Verify scope stack behavior**: Ensure `lookup_variables()` searches from innermost to outermost correctly
2. **Verify nested loop scoping**: Test that inner loop variables shadow outer loop variables
3. **Fix any scope management issues**: Ensure `enter_scope()`/`exit_scope()` are called at the right times
4. **Test nested loop variable shadowing**: Ensure variables with same name in nested loops work correctly

## Implementation Steps

1. **Review scope stack implementation** in `context.rs`:
   - Verify `lookup_variables()` searches scopes correctly
   - Verify `declare_variable()` adds to current scope correctly
   - Verify `enter_scope()`/`exit_scope()` manage scope stack correctly

2. **Review nested loop implementations**:
   - Verify for loops properly manage scope for nested loops
   - Verify while loops properly manage scope for nested loops
   - Verify do-while loops properly manage scope for nested loops (after Phase 2a)

3. **Test nested loop scenarios**:
   ```bash
   scripts/glsl-filetests.sh control/loop_for/nested.glsl
   scripts/glsl-filetests.sh control/loop_while/nested.glsl
   scripts/glsl-filetests.sh control/loop_do_while/nested.glsl
   scripts/glsl-filetests.sh control/nested/complex.glsl
   ```

4. **Fix any issues found**:
   - Adjust scope management if needed
   - Fix variable lookup if needed
   - Ensure proper scope nesting

## Files to Review/Modify

- `lightplayer/crates/lp-glsl/src/codegen/context.rs` - Verify scope stack implementation
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_for.rs` - Verify nested scope handling
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_while.rs` - Verify nested scope handling
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Verify nested scope handling (after Phase 2a)

## Test Cases

- `control/loop_for/nested.glsl` - Nested for loops
- `control/loop_while/nested.glsl` - Nested while loops
- `control/loop_do_while/nested.glsl` - Nested do-while loops
- `control/nested/complex.glsl` - Complex nested control flow
- `control/edge_cases/variable-shadowing.glsl` - Variable shadowing tests (especially nested cases)

## Expected Behavior

- Inner loop variables shadow outer loop variables correctly
- Variables from outer loops are accessible in inner loops (if not shadowed)
- Variables declared in loop bodies are scoped correctly
- Nested loops with same variable names work correctly
- All nested loop tests pass

## Verification

Run all nested loop tests:

```bash
scripts/glsl-filetests.sh control/loop_for/nested.glsl
scripts/glsl-filetests.sh control/loop_while/nested.glsl
scripts/glsl-filetests.sh control/loop_do_while/nested.glsl
scripts/glsl-filetests.sh control/nested/complex.glsl
```

Expected result: All nested loop tests pass.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix nested loop scope management"
```

## Notes

- This should be done after Phase 2a/2b (do-while fixes) and Phase 2c (test expectations)
- The scope stack implementation should already handle this, but needs verification
- May require debugging to understand why nested loops are failing
- Could be related to block sealing issues (Phase 1) if failures are SSA-related

