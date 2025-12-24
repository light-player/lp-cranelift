# Control Flow Fixes - Overview

This directory contains plans for fixing control flow compilation issues, organized by underlying root causes. These fixes align with proven patterns from DirectXShaderCompiler/Clang and address SSA dominance violations, block sealing issues, and variable scope handling.

## Problem Statement

The current control flow implementation has several critical issues:

1. **Block Sealing Violations**: Blocks sealed before all predecessors are declared, causing "assertion failed: !self.is_sealed(block)" panics
2. **SSA Dominance Violations**: Variables modified in different control flow paths lack proper phi nodes, causing "uses value from non-dominating block" verifier errors
3. **Variable Scope Issues**: Variable shadowing in nested scopes not properly handled, causing incorrect variable resolution
4. **Loop Variable Preservation**: Loop variables not correctly preserved after loop exit when modified in body
5. **Type Handling**: Boolean values (i8) incorrectly used as integers (i32) in some control flow paths
6. **Missing Features**: Logical OR operator (`||`) not yet implemented

## Current Test Status

**18 failing tests** across control flow categories (down from 38 after Phase 2):

- Edge cases: break-continue-edge-cases, condition-expressions, loop-expression-scope, non-terminating
- Loop types: do-while (basic, nested, variable scope), for/while (nested, complex conditions, variable scope)
- Break/continue: do-while-loop (break/continue)
- Nested control flow: complex combinations

**Remaining failures likely due to:**

- Phase 1 (block sealing) - do-while loops, break/continue
- Phase 2a/2b (do-while fixes) - scope and condition handling
- Phase 2c (test expectations) - incorrect expected values
- Phase 2d (nested loops) - scope management issues
- Phase 2e (complex conditions) - logical operators, condition evaluation
- Phase 6 (logical OR) - missing `||` operator

## Reference Architecture: DirectXShaderCompiler/Clang

We're aligning with Clang's proven control flow patterns:

### Key Patterns

1. **Block Management**:

   - Blocks created before branching
   - Blocks sealed only after all predecessors are known
   - Loop headers receive back edges, so sealed after loop body is emitted
   - Merge blocks sealed after all branches converge

2. **SSA Form**:

   - Variables modified in different paths require phi nodes at merge points
   - Cranelift's FunctionBuilder automatically creates phi nodes via `use_var` in correct block context
   - Variables read after control flow must be read in merge block, not before branching

3. **Variable Scope**:

   - Variables declared in inner scopes shadow outer variables
   - Variable lookups must respect scope hierarchy
   - Variables go out of scope when leaving their declaration block

4. **Loop Structure**:
   - While: header → body → back to header
   - Do-while: body → header → back to body or exit
   - For: init → header → body → update → back to header

### Example: Do-While Loop Pattern

```cpp
// Clang's pattern:
void CodeGenFunction::EmitDoStmt(const DoStmt &S) {
  // 1. Create blocks
  BasicBlock *BodyBlock = createBasicBlock("do.body");
  BasicBlock *CondBlock = createBasicBlock("do.cond");
  BasicBlock *ExitBlock = createBasicBlock("do.end");

  // 2. Push loop context for break/continue
  BreakContinueStack.push_back(BreakContinue(ExitBlock, CondBlock));

  // 3. Branch to body (do-while always executes once)
  EmitBranch(BodyBlock);

  // 4. Emit body
  EmitBlock(BodyBlock);
  EmitStmt(S.getBody());
  EmitBranch(CondBlock);

  // 5. Emit condition block
  EmitBlock(CondBlock);
  llvm::Value *CondValue = EvaluateExprAsBool(S.getCond());
  Builder.CreateCondBr(CondValue, BodyBlock, ExitBlock);

  // 6. Seal condition block (all predecessors known: initial branch + back edge)
  // Note: BodyBlock was sealed when we emitted it, but that's OK because
  // we declared it as a successor before sealing

  // 7. Emit exit block
  EmitBlock(ExitBlock);

  // 8. Pop loop context
  BreakContinueStack.pop_back();
}
```

## Root Cause Analysis

### Issue 1: Block Sealing Violations (Most Common)

**Problem**: In do-while loops, `emit_block(body_block)` seals the body block, but then `emit_cond_branch(..., body_block, ...)` tries to declare body_block as a successor, which fails because sealed blocks can't receive new predecessors.

**Affected**: All do-while loop tests, break/continue in do-while loops

**Solution**: Don't seal body_block until after header block declares it as successor. Use `switch_to_block` instead of `emit_block` for blocks that receive back edges.

### Issue 2: SSA Dominance Violations

**Problem**: When variables are modified in different control flow paths (e.g., nested if statements), reading the variable after the merge point tries to use a value from a non-dominating block.

**Example**:

```
block1: if (cond1) {
  block2: if (cond2) {
    x = 10;
  }
  block3: // tries to use x from block2, but block2 doesn't dominate block3
}
```

**Solution**: Ensure variables are read in the correct merge block context. Cranelift's `use_var` automatically creates phi nodes when called in a block with multiple predecessors.

### Issue 3: Variable Shadowing

**Problem**: Variables declared in inner scopes shadow outer variables, but the codegen doesn't properly track scope hierarchy.

**Solution**: Implement proper scope tracking in variable lookup, ensuring inner scope variables are found before outer scope variables.

### Issue 4: Loop Variable Preservation

**Problem**: For loop update expressions execute even when loop variable is modified in body, causing double increment.

**Solution**: Ensure loop variable reads/writes happen in correct blocks, and update expression uses the correct variable value.

### Issue 5: Type Errors

**Problem**: Boolean comparison results (i8) incorrectly used as integers (i32) in arithmetic operations.

**Solution**: Ensure proper type conversion when using boolean values in expressions.

## Fix Phases

### Phase 01a: Fix Compound Statement Scoping

**Goal**: Ensure compound statements (`{ ... }`) properly create and exit variable scopes

- Add `enter_scope()`/`exit_scope()` calls to compound statements
- Match semantic analysis behavior (semantic analysis already creates scopes)
- Fix fundamental scoping issue affecting all `{ }` blocks
- Should be done first as it's foundational

### Phase 1: Fix Do-While Loop Block Sealing and SSA Issues

**Goal**: Fix block sealing order and SSA value handling to prevent "index out of bounds" errors

- Fix block sealing order in do-while loops
- Ensure variables used in condition block are properly resolved
- Prevent SSA from creating invalid value references
- See `01-fix-do-while-block-sealing.md` for detailed analysis and implementation

### Phase 2: Fix SSA Dominance Violations ✅ COMPLETE

**Goal**: Ensure variables modified in control flow paths have proper phi nodes

- Fix variable reading to happen in correct merge blocks
- Ensure `use_var` is called in blocks with proper dominance
- Fix nested control flow variable handling

### Phase 2a: Fix Do-While Loop Scope Management

**Goal**: Add proper scope management to do-while loops

- Add `enter_scope()`/`exit_scope()` calls to do-while loop bodies
- Ensure variables declared in do-while bodies are properly scoped
- Fix variable shadowing in do-while loops

### Phase 2b: Fix Do-While Loop Condition Type Handling

**Goal**: Ensure do-while conditions are properly converted to boolean type

- Use proper type validation for do-while conditions
- Match the pattern used in while loops
- Ensure GLSL spec compliance (conditions must be bool type)

### Phase 2c: Fix Test Expectations for Variable Shadowing

**Goal**: Correct test expectations that have incorrect expected values

- Fix `test_if_variable_shadowing` expectation (10 → 5)
- Fix `test_for_loop_init_shadowing` expectation (3 → 100)
- Fix `test_while_loop_shadowing` expectation (20 → 100)

### Phase 2d: Fix Nested Loop Scope Management

**Goal**: Ensure nested loops properly manage variable scopes

- Verify scope stack handles nested scopes correctly
- Fix inner loop variables shadowing outer loop variables
- Ensure proper scope nesting for all loop types

### Phase 2e: Fix Complex Condition Expressions

**Goal**: Ensure complex condition expressions work correctly

- Fix logical AND (`&&`) operator in conditions
- Fix condition evaluation for complex expressions
- Note: Logical OR (`||`) is Phase 6 (separate feature)

### Phase 3: Fix Variable Shadowing ✅ COMPLETE (merged into Phase 2)

**Goal**: Properly handle variable shadowing in nested scopes

- Implement scope tracking in variable lookup
- Ensure inner scope variables shadow outer scope variables correctly
- Fix variable resolution to respect scope hierarchy

### Phase 4: Fix Loop Variable Preservation

**Goal**: Ensure loop variables are correctly preserved after loop exit

- Fix for loop update expression handling
- Ensure loop variable reads happen in correct blocks
- Fix variable preservation when modified in loop body

### Phase 5: Fix Type Handling in Control Flow

**Goal**: Ensure boolean values are properly handled in control flow

- Fix type conversion for boolean values used in expressions
- Ensure comparison results are properly typed

### Phase 6: Implement Logical OR Operator (Separate Feature)

**Goal**: Implement missing `||` operator

- Add logical OR operator support
- Handle short-circuit evaluation
- Test with complex condition expressions

## Test Commands

### Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

### Run specific test category:

```bash
# Do-while loops
scripts/glsl-filetests.sh control/loop_do_while/

# Break/continue
scripts/glsl-filetests.sh control/loop_break/
scripts/glsl-filetests.sh control/loop_continue/

# Edge cases
scripts/glsl-filetests.sh control/edge_cases/

# Nested control flow
scripts/glsl-filetests.sh control/nested/
```

### Run specific test file:

```bash
scripts/glsl-filetests.sh control/loop_do_while/basic.glsl
```

### Run specific test case:

```bash
scripts/glsl-filetests.sh control/edge_cases/variable-shadowing.glsl:47
```

## Dependencies

- Phase 01a (compound scoping) - foundational fix, should be done first
- Phase 1 (block sealing) - should be done after Phase 01a - fixes most common failures
- Phase 2 (SSA dominance) ✅ COMPLETE - depends on Phase 01a/Phase 1 - needs correct block structure
- Phase 2a (do-while scope) - should be done after Phase 2, before Phase 2b
- Phase 2b (do-while condition) - depends on Phase 2a
- Phase 2c (test expectations) - can be done after Phase 2, verifies correctness
- Phase 2d (nested loops) - depends on Phase 01a, Phase 2a/2b, may reveal Phase 1 issues
- Phase 2e (complex conditions) - depends on Phase 2b, some tests require Phase 6
- Phase 3 (variable shadowing) ✅ COMPLETE - merged into Phase 2
- Phase 4 (loop variables) - depends on Phase 2 - needs proper phi nodes
- Phase 5 (type handling) - can be done independently
- Phase 6 (logical OR) - separate feature, required for some Phase 2e tests

## Commit Instructions

After completing each phase:

1. **Verify tests pass:**

   ```bash
   scripts/glsl-filetests.sh control/
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
- **Align with Clang** - follow proven patterns from Clang's codegen
- **Do it right** - take time to understand and implement correctly
- **Test frequently** - run tests after each change to catch regressions early
