# Phase 3: Improve Block Management

## Goal

Add explicit block context tracking and ensure expressions are always evaluated in the correct block context, following Clang's pattern of explicit block management.

## Current Issues

1. **Implicit Block Context**: No explicit tracking of which block we're in
2. **Expression Evaluation**: Expressions may be evaluated in wrong block
3. **Variable Reading**: Variables may be read before switching to correct block
4. **No Assertions**: No way to catch block context issues early

## Target Architecture (Clang Pattern)

### Block Management Methods

Clang uses explicit block management:
- `EmitBlock(block)` - Switch to block and emit code
- `EmitBranch(target)` - Branch to target block
- `Builder.GetInsertBlock()` - Get current insertion point
- `Builder.SetInsertPoint(block)` - Set insertion point

### Block Context Tracking

```rust
impl CodegenContext {
    /// Get current block
    pub fn current_block(&self) -> Option<Block> {
        self.builder.current_block()
    }
    
    /// Ensure we're in a block before evaluating expressions
    pub fn ensure_block(&mut self) -> Result<Block, GlslError> {
        self.builder.current_block()
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "not in a block"))
    }
    
    /// Switch to block and seal it
    pub fn emit_block(&mut self, block: Block) {
        self.builder.switch_to_block(block);
        self.builder.seal_block(block);
    }
    
    /// Create and emit block
    pub fn create_and_emit_block(&mut self, name: &str) -> Block {
        let block = self.builder.create_block();
        self.emit_block(block);
        block
    }
    
    /// Branch to target block
    pub fn emit_branch(&mut self, target: Block) {
        self.builder.ins().jump(target, &[]);
    }
    
    /// Conditional branch
    pub fn emit_cond_branch(&mut self, cond: Value, then_block: Block, else_block: Block) {
        self.builder.ins().brif(cond, then_block, &[], else_block, &[]);
    }
}
```

## Implementation Steps

### Step 1: Add Block Context Methods

1. Add helper methods to `CodegenContext`:
   - `current_block()` - Get current block
   - `ensure_block()` - Ensure we're in a block
   - `emit_block()` - Switch to and seal block
   - `create_and_emit_block()` - Create, switch to, and seal block
   - `emit_branch()` - Branch to block
   - `emit_cond_branch()` - Conditional branch

2. Add assertions:
   - Assert we're in a block before reading variables
   - Assert we're in a block before evaluating expressions
   - Assert block context is correct

### Step 2: Refactor Statement Translation

1. Update `translate_selection`:
   - Use explicit block management methods
   - Ensure condition evaluated in correct block
   - Ensure branches created and switched to correctly
   - Follow Clang's `EmitIfStmt` pattern exactly

2. Update `translate_iteration`:
   - Use explicit block management
   - Ensure loop header, body, update blocks managed correctly

### Step 3: Refactor Expression Evaluation

1. Update `emit_rvalue`:
   - Assert we're in a block before evaluating
   - Ensure operands evaluated in correct block
   - Don't evaluate expressions before switching blocks

2. Update variable reading:
   - Assert we're in correct block before reading
   - Ensure `use_var` called in correct block context
   - Don't cache values across blocks

### Step 4: Add Block Context Verification

1. Add debug assertions:
   - Verify block context before variable reads
   - Verify block context before expression evaluation
   - Verify block context before branching

2. Add logging (optional):
   - Log block switches
   - Log variable reads with block context
   - Log expression evaluation with block context

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/context.rs` - Add block management methods
- `lightplayer/crates/lp-glsl/src/codegen/stmt.rs` - Refactor to use explicit block management
- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Add block context assertions
- `lightplayer/crates/lp-glsl/src/codegen/expr/variable.rs` - Add block context verification
- `lightplayer/crates/lp-glsl/src/codegen/expr/component.rs` - Add block context verification

## Reference: Clang's If Statement Pattern

```cpp
void CodeGenFunction::EmitIfStmt(const IfStmt &S) {
  // 1. Create blocks
  llvm::BasicBlock *ThenBlock = createBasicBlock("if.then");
  llvm::BasicBlock *ContBlock = createBasicBlock("if.end");
  llvm::BasicBlock *ElseBlock = ContBlock;
  if (S.getElse())
    ElseBlock = createBasicBlock("if.else");

  // 2. Evaluate condition and branch (in current block)
  EmitBranchOnBoolExpr(S.getCond(), ThenBlock, ElseBlock);

  // 3. Emit then block
  EmitBlock(ThenBlock);  // Switch to then block
  EmitStmt(S.getThen());
  EmitBranch(ContBlock);  // Branch to continuation

  // 4. Emit else block (if present)
  if (S.getElse()) {
    EmitBlock(ElseBlock);  // Switch to else block
    EmitStmt(S.getElse());
    EmitBranch(ContBlock);  // Branch to continuation
  }

  // 5. Emit continuation block
  EmitBlock(ContBlock);  // Switch to continuation block
}
```

## Test Cases

- All existing tests should continue to pass
- Verify block context is correct
- Test nested conditionals
- Test loops

## Acceptance Criteria

- [ ] Block management methods added
- [ ] Block context assertions added
- [ ] Statement translation uses explicit block management
- [ ] Expression evaluation verifies block context
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Clear block context tracking

## Verification

Run all tests:
```bash
scripts/glsl-filetests.sh vec4/
```

Expected result: All tests pass, block context is explicit and correct.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: improve block management with explicit context tracking"
```

