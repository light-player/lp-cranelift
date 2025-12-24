# Phase 4: Refactor Statement Translation

## Goal

Refactor statement translation to exactly match Clang's patterns, ensuring proper control flow handling and block management.

## Current Issues

1. **If Statement Pattern**: Doesn't exactly match Clang's pattern
2. **Block Management**: May not handle all edge cases correctly
3. **Control Flow**: May not handle complex control flow correctly

## Target Architecture (Clang Pattern)

### If Statement Pattern

```cpp
void CodeGenFunction::EmitIfStmt(const IfStmt &S) {
  // 1. Handle constant folding (optional optimization)
  if (ConstantFoldsToSimpleInteger(S.getCond(), CondConstant)) {
    // Emit only the executed branch
    const Stmt *Executed = CondConstant ? S.getThen() : S.getElse();
    if (Executed) {
      EmitStmt(Executed);
    }
    return;
  }

  // 2. Create blocks
  llvm::BasicBlock *ThenBlock = createBasicBlock("if.then");
  llvm::BasicBlock *ContBlock = createBasicBlock("if.end");
  llvm::BasicBlock *ElseBlock = S.getElse() ? createBasicBlock("if.else") : ContBlock;

  // 3. Evaluate condition and branch (in current block)
  EmitBranchOnBoolExpr(S.getCond(), ThenBlock, ElseBlock);

  // 4. Emit then block
  EmitBlock(ThenBlock);
  {
    RunCleanupsScope ThenScope(*this);
    EmitStmt(S.getThen());
  }
  EmitBranch(ContBlock);

  // 5. Emit else block (if present)
  if (S.getElse()) {
    EmitBlock(ElseBlock);
    {
      RunCleanupsScope ElseScope(*this);
      EmitStmt(S.getElse());
    }
    EmitBranch(ContBlock);
  }

  // 6. Emit continuation block
  EmitBlock(ContBlock);
}
```

### While Statement Pattern

```cpp
void CodeGenFunction::EmitWhileStmt(const WhileStmt &S) {
  // 1. Create blocks
  llvm::BasicBlock *HeaderBlock = createBasicBlock("while.cond");
  llvm::BasicBlock *BodyBlock = createBasicBlock("while.body");
  llvm::BasicBlock *ExitBlock = createBasicBlock("while.end");

  // 2. Push loop context
  BreakContinueStack.push_back(BreakContinue(ExitBlock, HeaderBlock));

  // 3. Branch to header
  EmitBranch(HeaderBlock);

  // 4. Emit header (condition evaluation)
  EmitBlock(HeaderBlock);
  llvm::Value *CondValue = EvaluateExprAsBool(S.getCond());
  Builder.CreateCondBr(CondValue, BodyBlock, ExitBlock);

  // 5. Emit body
  EmitBlock(BodyBlock);
  {
    RunCleanupsScope BodyScope(*this);
    EmitStmt(S.getBody());
  }
  EmitBranch(HeaderBlock);  // Loop back

  // 6. Emit exit block
  EmitBlock(ExitBlock);

  // 7. Pop loop context
  BreakContinueStack.pop_back();
}
```

## Implementation Steps

### Step 1: Refactor If Statement

1. Update `translate_selection`:

   - Match Clang's `EmitIfStmt` pattern exactly
   - Create blocks before branching
   - Evaluate condition in current block
   - Switch to blocks explicitly
   - Branch explicitly

2. Handle constant folding (optional):
   - Check if condition is constant
   - Emit only executed branch if constant

### Step 2: Refactor While Statement

1. Update `translate_iteration` for while loops:
   - Match Clang's `EmitWhileStmt` pattern
   - Create header, body, exit blocks
   - Push/pop loop context
   - Handle break/continue correctly

### Step 3: Refactor For Statement

1. Update `translate_iteration` for for loops:
   - Match Clang's `EmitForStmt` pattern
   - Create init, header, body, update, exit blocks
   - Handle continue jumping to update block
   - Handle break jumping to exit block

### Step 4: Refactor Other Statements

1. Update `translate_return`:

   - Match Clang's `EmitReturnStmt` pattern
   - Handle return value correctly

2. Update `translate_compound`:
   - Match Clang's `EmitCompoundStmt` pattern
   - Handle statement sequence correctly

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt.rs` - Refactor all statement translation
- `lightplayer/crates/lp-glsl/src/codegen/context.rs` - Add loop context management if needed

## Reference Implementation

Study Clang's implementation:

- `CGStmt.cpp` - `EmitIfStmt`, `EmitWhileStmt`, `EmitForStmt`
- `CodeGenFunction.h` - Method signatures and patterns

## Test Cases

- All existing tests should continue to pass
- Test nested conditionals
- Test nested loops
- Test break/continue
- Test return statements

## Acceptance Criteria

- [ ] If statements match Clang pattern exactly
- [ ] While statements match Clang pattern exactly
- [ ] For statements match Clang pattern exactly
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Control flow is correct

## Verification

Run all matrix tests:

```bash
scripts/glsl-filetests.sh matrix/
```

Expected result: All tests pass, control flow matches Clang patterns.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: refactor statement translation to match Clang patterns"
```
