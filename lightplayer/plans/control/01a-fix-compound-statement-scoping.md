# Phase 01a: Fix Compound Statement Scoping

## Goal

Ensure compound statements (`{ ... }`) properly create and exit variable scopes during code generation, matching the behavior of semantic analysis.

## Problem

Compound statements are not creating scopes during code generation, even though semantic analysis correctly creates scopes for them. This causes:

- Variables declared in `{ }` blocks not being properly scoped
- Variable shadowing not working correctly in arbitrary blocks
- Inconsistency between semantic analysis and code generation
- Variables leaking from inner scopes to outer scopes incorrectly

**Current Code** (missing scopes):

```rust
pub fn emit_compound_stmt(
    ctx: &mut CodegenContext,
    compound: &CompoundStatement,
) -> Result<(), GlslError> {
    for stmt in &compound.statement_list {
        ctx.emit_statement(stmt)?;  // ❌ NO SCOPE MANAGEMENT!
    }
    Ok(())
}
```

**Semantic Analysis** (correct):

```rust
Statement::Compound(compound) => {
    symbols.push_scope();  // ✅ Creates scope
    for stmt in &compound.statement_list {
        validate_statement(stmt, symbols, ...)?;
    }
    symbols.pop_scope();  // ✅ Exits scope
}
```

## Root Cause

In C/GLSL, every `{ }` block creates a new scope. Variables declared inside `{ }` should:

- Shadow outer variables with the same name
- Go out of scope when the block ends
- Not be accessible after the block closes

Currently, we only create scopes for:

- If statement bodies (manually added in Phase 2)
- Loop bodies (manually added in Phase 2)
- But NOT for standalone `{ }` blocks

## Solution

Add scope management to `emit_compound_stmt` to match semantic analysis:

```rust
pub fn emit_compound_stmt(
    ctx: &mut CodegenContext,
    compound: &CompoundStatement,
) -> Result<(), GlslError> {
    ctx.enter_scope();  // Enter scope for compound block
    for stmt in &compound.statement_list {
        ctx.emit_statement(stmt)?;
    }
    ctx.exit_scope();  // Exit scope for compound block
    Ok(())
}
```

## Why This Matters

This is fundamental to C/GLSL scoping rules. Examples:

```glsl
int x = 5;
{
    int x = 10;  // Shadows outer x
    // x is 10 here
}
// x is 5 here (outer x)
```

Without this fix, shadowing won't work correctly in arbitrary blocks, not just control flow constructs.

## Implementation Steps

1. **Update `emit_compound_stmt` in `compound.rs`**:

   - Add `ctx.enter_scope()` before processing statements
   - Add `ctx.exit_scope()` after processing statements
   - Ensure scope is properly managed even if statements fail

2. **Test with compound statement tests**:

   ```bash
   scripts/glsl-filetests.sh control/edge_cases/compound-statements.glsl
   ```

3. **Verify no regressions**:
   - All existing tests should still pass
   - Variable shadowing should work correctly in blocks

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt/compound.rs` - Add scope management

## Test Cases

- `control/edge_cases/compound-statements.glsl` - Compound statement tests
- `control/edge_cases/variable-shadowing.glsl` - Variable shadowing tests (especially nested blocks)
- All control flow tests that use blocks

## Expected Behavior

- Every `{ }` block creates a new scope
- Variables declared in `{ }` blocks shadow outer variables
- Variables declared in `{ }` blocks go out of scope when block ends
- Semantic analysis and code generation are consistent
- All existing tests continue to pass

## Verification

Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

Expected result: More tests pass, especially those involving variable shadowing in blocks.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: add scope management to compound statements"
```

## Notes

- This is a fundamental fix that should be done before other scoping fixes
- It's a simple 2-line change but has wide-reaching impact
- This ensures consistency between semantic analysis and code generation
- This is independent of block sealing issues (Phase 1)
- This should be done before Phase 2a-2e (loop-specific scoping fixes)

## Dependencies

- None - this is a foundational fix
- Should be done before Phase 1 (block sealing) as it affects all scoping
- Other phases may depend on this being correct
