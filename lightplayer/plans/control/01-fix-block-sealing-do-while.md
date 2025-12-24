# Phase 1: Fix Block Sealing in Do-While Loops

## Goal

Fix block sealing order so do-while loops don't panic with "assertion failed: !self.is_sealed(block)" errors. This is the most common failure affecting all do-while loop tests.

## Problem

In `emit_loop_do_while_stmt`, the body block is sealed too early:

1. Line 25: `ctx.emit_block(body_block)` - This **seals** the body_block
2. Line 27: `ctx.emit_branch(header_block)` - Branches to header
3. Line 33: `ctx.emit_cond_branch(condition_value, body_block, exit_block)` - Tries to declare body_block as a successor, but it's already sealed!

**Error**: Cranelift's SSA builder doesn't allow declaring successors to sealed blocks. The assertion `!self.is_sealed(block)` fails at `cranelift/frontend/src/ssa.rs:374`.

## Root Cause

The issue is that `emit_block()` seals the block immediately, but in do-while loops:

- The body_block needs to receive a back edge from the header_block
- The header_block declares body_block as a successor via `emit_cond_branch`
- But body_block was already sealed before the header could declare it as a successor

## Solution

Don't seal body_block until after the header block has declared it as a successor. Use `switch_to_block` instead of `emit_block` for blocks that receive back edges.

### Correct Pattern (from Clang)

```cpp
// 1. Create blocks (don't seal yet)
BasicBlock *BodyBlock = createBasicBlock("do.body");
BasicBlock *CondBlock = createBasicBlock("do.cond");
BasicBlock *ExitBlock = createBasicBlock("do.end");

// 2. Branch to body
EmitBranch(BodyBlock);

// 3. Emit body (switch to but don't seal - will receive back edge)
SetInsertPoint(BodyBlock);
EmitStmt(S.getBody());
EmitBranch(CondBlock);

// 4. Emit condition block (switch to but don't seal - will receive back edge)
SetInsertPoint(CondBlock);
Value *CondValue = EvaluateExprAsBool(S.getCond());
Builder.CreateCondBr(CondValue, BodyBlock, ExitBlock);
// Now BodyBlock is declared as successor - safe to seal

// 5. Seal blocks (all predecessors known)
SealBlock(BodyBlock);
SealBlock(CondBlock);

// 6. Emit exit block (seal immediately - all predecessors known)
EmitBlock(ExitBlock);
```

## Implementation Steps

### Step 1: Fix `emit_loop_do_while_stmt`

1. Update `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs`:

   ```rust
   pub fn emit_loop_do_while_stmt(
       ctx: &mut CodegenContext,
       body: &Statement,
       condition: &Expr,
   ) -> Result<(), GlslError> {
       let body_block = ctx.builder.create_block();
       let header_block = ctx.builder.create_block();
       let exit_block = ctx.builder.create_block();

       ctx.loop_stack.push(crate::codegen::context::LoopContext {
           continue_target: header_block,
           exit_block,
       });

       // Jump directly to body (do-while always executes once)
       ctx.emit_branch(body_block)?;

       // Body: switch to but don't seal yet - will receive back edge from header
       ctx.switch_to_block(body_block);
       ctx.emit_statement(body)?;
       ctx.emit_branch(header_block)?;

       // Header: evaluate condition
       // Don't seal header yet - it will receive a back edge from body
       ctx.switch_to_block(header_block);
       let condition_value = ctx.translate_expr(condition)?;
       ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
       // Now body_block is declared as successor - safe to seal

       // Seal blocks now that all predecessors are known
       ctx.seal_block(body_block);
       ctx.seal_block(header_block);

       // Exit - seal immediately since all predecessors are known
       ctx.emit_block(exit_block);

       ctx.loop_stack.pop();

       Ok(())
   }
   ```

2. Key changes:
   - Line 25: Change `ctx.emit_block(body_block)` to `ctx.switch_to_block(body_block)`
   - Line 36: Add `ctx.seal_block(body_block)` after header declares it as successor
   - Line 37: Add `ctx.seal_block(header_block)` after all predecessors known

### Step 2: Verify Break/Continue Handling

Ensure break/continue statements work correctly:

- Break should branch to `exit_block` (already handled)
- Continue should branch to `header_block` (already handled)
- Both should create unreachable blocks after branching (already handled)

### Step 3: Test

Run all do-while loop tests:

```bash
scripts/glsl-filetests.sh control/loop_do_while/
scripts/glsl-filetests.sh control/loop_break/do-while-loop.glsl
scripts/glsl-filetests.sh control/loop_continue/do-while-loop.glsl
scripts/glsl-filetests.sh control/edge_cases/non-terminating.glsl
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Fix block sealing order

## Test Cases

All do-while loop tests should pass:

- `control/loop_do_while/basic.glsl` - Basic do-while loops
- `control/loop_do_while/nested.glsl` - Nested do-while loops
- `control/loop_do_while/runs-at-least-once.glsl` - Do-while always executes once
- `control/loop_do_while/variable-scope.glsl` - Variable scope in do-while
- `control/loop_break/do-while-loop.glsl` - Break in do-while
- `control/loop_continue/do-while-loop.glsl` - Continue in do-while
- `control/edge_cases/non-terminating.glsl` - Non-terminating loops

## Expected Behavior

- Do-while loops compile without panics
- Break statements correctly exit the loop
- Continue statements correctly jump to condition check
- Loop variables are preserved correctly after loop exit

## Verification

Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

Expected result: All do-while loop tests pass, no block sealing panics.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix block sealing order in do-while loops"
```

## Notes

- This fix follows the same pattern as while loops (which already work correctly)
- The key insight is that blocks receiving back edges must not be sealed until after the back edge is declared
- Cranelift's SSA builder enforces this with assertions to ensure correct SSA form
