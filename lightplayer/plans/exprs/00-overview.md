# Expression Compilation Architecture Refactoring - Overview

This directory contains plans for refactoring the expression compilation architecture to align with proven patterns from DirectXShaderCompiler/Clang, fixing SSA dominance violations and improving maintainability.

## Problem Statement

The current expression compilation architecture has several critical issues:

1. **SSA Dominance Violations**: Values computed in one branch are incorrectly used in subsequent code, causing "uses value from non-dominating inst" verifier errors
2. **Mixed Concerns**: Expression evaluation mixes LValue and RValue concerns in a single `translate_expr_typed` method
3. **Block Management**: No clear separation between when expressions are evaluated vs when blocks are created
4. **Value Reuse Issues**: Values may be incorrectly cached or reused across different basic blocks

## Reference Architecture: DirectXShaderCompiler/Clang

We're aligning with Clang's proven architecture:

### Key Patterns

1. **LValue vs RValue Separation**:

   - `EmitLValue(expr)` - Returns `LValue` wrapper for modifiable locations
   - `EmitScalarExpr(expr)` / `EmitAggExpr(expr)` - Returns `RValue` wrapper for values
   - Clear separation ensures we don't mix concerns

2. **RValue Wrapper**:

   - `RValue` type wraps scalar/complex/aggregate uniformly
   - Handles different evaluation kinds (scalar, complex, aggregate)
   - Provides consistent interface for expression results

3. **Block Management**:

   - Builder manages insertion point (`Builder.GetInsertBlock()`, `Builder.SetInsertPoint()`)
   - Expressions evaluated in current insertion point
   - Conditions evaluated before branching, but variable reads happen in correct block

4. **Expression Visitor Pattern**:
   - Each expression type has dedicated handler
   - Clear separation of concerns
   - Easier to extend and maintain

### Example: If Statement Pattern

```cpp
// Clang's pattern:
void CodeGenFunction::EmitIfStmt(const IfStmt &S) {
  // 1. Evaluate condition in current block
  EmitBranchOnBoolExpr(S.getCond(), ThenBlock, ElseBlock);

  // 2. Emit then block
  EmitBlock(ThenBlock);
  EmitStmt(S.getThen());
  EmitBranch(ContBlock);

  // 3. Emit else block (if present)
  if (S.getElse()) {
    EmitBlock(ElseBlock);
    EmitStmt(S.getElse());
    EmitBranch(ContBlock);
  }

  // 4. Emit continuation block
  EmitBlock(ContBlock);
}
```

## Refactoring Phases

### Phase 1: Restructure Statement Module

**Goal**: Break out `stmt.rs` into clean module structure with per-statement files

- Create `stmt/` module directory
- Extract each statement type to its own file
- Rename functions to `emit_*_stmt` pattern (aligning with Clang)
- Organize helpers appropriately
- Make functions public for clarity and testing

### Phase 2: Introduce RValue/LValue Separation

**Goal**: Separate LValue and RValue evaluation paths

- Create `RValue` wrapper type
- Create `EmitRValue` method separate from `EmitLValue`
- Refactor expression handlers to use appropriate method

### Phase 3: Improve Block Management

**Goal**: Explicit block context tracking and proper insertion point management

- Add explicit block context tracking
- Ensure expressions evaluated in correct blocks
- Add assertions to catch block context issues early

### Phase 4: Refactor Statement Implementation

**Goal**: Align statement implementation with Clang patterns

- Refactor `emit_if_stmt` to match Clang's `EmitIfStmt` pattern exactly
- Refactor loop implementations to match Clang patterns
- Ensure proper block creation and branching
- Fix control flow handling

### Phase 5: Extract Expression Helpers (Idiomatic Rust)

**Goal**: Improve maintainability with idiomatic Rust patterns

- Extract helper functions for complex and repeated logic
- Keep match statements (exhaustive, type-safe, idiomatic)
- Extract common "resolve LValue then load" pattern
- Make helpers public for testing
- Add documentation to all helpers

## Current Test Failures

These tests are failing due to SSA dominance violations:

- `vec4/indexing/array-indexing.glsl:43` - `test_vec4_array_indexing_equals_component()`
- `vec4/indexing/component-access.glsl:58` - `test_vec4_component_access_verify_synonyms()`

## Test Commands

### Run all vec4 tests:

```bash
scripts/glsl-filetests.sh vec4/
```

### Run specific test file:

```bash
scripts/glsl-filetests.sh vec4/indexing/array-indexing.glsl:43
```

### Run specific test case:

```bash
scripts/glsl-filetests.sh vec4/indexing/array-indexing.glsl:43
```

## Dependencies

- Phase 1 (restructure) should be done first - provides clean foundation
- Phase 2 (RValue/LValue) depends on Phase 1
- Phase 3 (block management) depends on Phase 2
- Phase 4 (statement implementation) depends on Phase 3 (can leverage new structure)
- Phase 5 (extract helpers) is a pure refactoring and can be done independently after Phase 2

## Commit Instructions

After completing each phase:

1. **Verify all tests pass:**

   ```bash
   scripts/glsl-filetests.sh vec4/
   ```

2. **Commit with appropriate message:**

   ```bash
   git add -A
   git commit -m "lpc: [phase description]"
   ```

3. **Keep commits small and focused** - one logical change per commit

## Notes

- **No backwards compatibility required** - we're willing to totally rework code
- **Delete problematic parts** - don't try to fix, replace with correct implementation
- **Align with DXC** - follow proven patterns from DirectXShaderCompiler
- **Do it right** - take time to understand and implement correctly
