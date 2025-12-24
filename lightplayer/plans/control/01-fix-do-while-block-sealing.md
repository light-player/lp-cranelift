# Phase 1: Fix Do-While Loop Block Sealing and SSA Issues

## Goal

Fix block sealing order and SSA value handling in do-while loops to prevent "index out of bounds" errors during verification. The error occurs when the verifier tries to access a Value that doesn't exist, indicating malformed IR due to incorrect block sealing or variable usage across blocks.

## Problem

Do-while loops fail with "index out of bounds: the len is 13 but the index is 13" errors in `dfg.rs:391` during verification. This indicates:

1. **Invalid Value References**: The verifier is trying to access a Value (index 13) that doesn't exist in the DFG
2. **Block Sealing Order**: Blocks may be sealed in the wrong order, causing SSA to create invalid value references
3. **Variable Usage Across Blocks**: Variables modified in the body block and used in the condition block may not have proper phi nodes

**Error Location**: `cranelift/codegen/src/ir/dfg.rs:391` in `value_def()` - trying to access a Value that doesn't exist

## Root Cause Analysis

### Key Difference: While vs Do-While

**While loops** (working):
- Condition evaluated BEFORE body
- Variables used in condition come from before loop
- Body block sealed immediately after emitting (no back edge yet)
- Header block sealed after body declares back edge

**Do-while loops** (broken):
- Condition evaluated AFTER body  
- Variables used in condition may be modified in body
- Body block receives back edge from condition block
- Both blocks need to remain unsealed until all predecessors are known

### The Problem

When variables are modified in the body block and then used in the condition block:

1. Condition block uses `use_var()` for variables modified in body
2. Since body block isn't sealed yet, SSA creates a block parameter for the condition block
3. When body block is sealed, it needs to pass values to condition block parameters
4. But if the sealing order is wrong, or if values are referenced incorrectly, we get invalid Value indices

### Specific Issues

1. **Variable Usage Before Block Sealing**: Variables used in condition block may reference values that don't exist yet
2. **Sealing Order**: Body block sealed before all variable uses are resolved
3. **SSA Phi Node Creation**: When body block is sealed, it needs to pass values to condition block, but the values might not be properly tracked

## Solution

### Pattern: Match Clang's Do-While Implementation

Clang's pattern (from overview):
```cpp
// 1. Create blocks
BasicBlock *BodyBlock = createBasicBlock("do.body");
BasicBlock *CondBlock = createBasicBlock("do.cond");
BasicBlock *ExitBlock = createBasicBlock("do.end");

// 2. Branch to body
EmitBranch(BodyBlock);

// 3. Emit body (seals immediately)
EmitBlock(BodyBlock);  // Seals body block
EmitStmt(S.getBody());
EmitBranch(CondBlock);

// 4. Emit condition block (seals immediately)
EmitBlock(CondBlock);  // Seals condition block
llvm::Value *CondValue = EvaluateExprAsBool(S.getCond());
Builder.CreateCondBr(CondValue, BodyBlock, ExitBlock);
// Note: BodyBlock was sealed when we emitted it, but that's OK because
// we declared it as a successor before sealing
```

**Key Insight**: In LLVM/Clang, you can seal a block and then later declare it as a successor. The block parameter creation happens when the block is sealed, and then the predecessor passes arguments when it branches.

### Our Current Implementation

```rust
// Body: switch to but don't seal yet
ctx.switch_to_block(body_block);
ctx.emit_statement(body)?;
ctx.emit_branch(cond_block)?;

// Condition: switch to but don't seal yet  
ctx.switch_to_block(cond_block);
let condition_value = ctx.translate_expr(condition)?;  // Uses variables!
ctx.emit_cond_branch(condition_value, body_block, exit_block)?;

// Now seal both blocks
ctx.seal_block(body_block);
ctx.seal_block(cond_block);
```

**Problem**: We're using variables in the condition block before sealing the body block. When SSA tries to resolve these variables, it may create invalid references.

### Fix Strategy

**Option 1: Seal Body Block Before Using Variables in Condition** (Recommended)

Seal the body block immediately after emitting it, before switching to the condition block. This ensures all variable definitions in the body are finalized before they're used in the condition.

```rust
// Body: emit and seal immediately
ctx.switch_to_block(body_block);
ctx.emit_statement(body)?;
ctx.emit_branch(cond_block)?;
ctx.seal_block(body_block);  // Seal BEFORE condition uses variables

// Condition: now safe to use variables from body
ctx.switch_to_block(cond_block);
let condition_value = ctx.translate_expr(condition)?;
ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
ctx.seal_block(cond_block);  // Seal after declaring back edge
```

**Option 2: Ensure Variables Are Read in Correct Block Context**

Ensure that when variables are used in the condition block, they're read in the condition block's context, allowing SSA to create proper phi nodes.

**Option 3: Match Clang Pattern Exactly**

Use `emit_block()` for body block (seals immediately), then use `emit_block()` for condition block (seals immediately). The back edge from condition to body is declared after body is sealed, which Clang says is OK.

## Implementation Steps

### Step 1: Understand Current Block Sealing Behavior

1. Check if `emit_block()` can be used instead of `switch_to_block()` + `seal_block()`
2. Verify that sealing body block before condition block won't break back edge handling
3. Test if sealing order affects SSA value creation

### Step 2: Fix Block Sealing Order

Update `emit_loop_do_while_stmt` in `loop_do_while.rs`:

**Approach A**: Seal body block before condition block
```rust
// Body: emit and seal immediately
ctx.switch_to_block(body_block);
ctx.enter_scope();
ctx.emit_statement(body)?;
ctx.exit_scope();
ctx.emit_branch(cond_block)?;
ctx.seal_block(body_block);  // Seal BEFORE condition

// Condition: now use variables
ctx.switch_to_block(cond_block);
let condition_value = ctx.translate_expr(condition)?;
ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
ctx.seal_block(cond_block);
```

**Approach B**: Use `emit_block()` like Clang
```rust
// Body: emit and seal immediately (like Clang)
ctx.emit_block(body_block);  // Seals immediately
ctx.enter_scope();
ctx.emit_statement(body)?;
ctx.exit_scope();
ctx.emit_branch(cond_block)?;

// Condition: emit and seal immediately
ctx.emit_block(cond_block);  // Seals immediately
let condition_value = ctx.translate_expr(condition)?;
ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
// Back edge declared after body is sealed - Clang says this is OK
```

### Step 3: Verify Variable Usage

1. Ensure variables modified in body are properly read in condition block
2. Verify phi nodes are created correctly when body block is sealed
3. Check that SSA doesn't create invalid value references

### Step 4: Test All Do-While Cases

```bash
scripts/glsl-filetests.sh control/loop_do_while/
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/stmt/loop_do_while.rs` - Fix block sealing order

## Test Cases

- `control/loop_do_while/basic.glsl` - Basic do-while loops
- `control/loop_do_while/nested.glsl` - Nested do-while loops
- `control/loop_do_while/variable-scope.glsl` - Variable scoping
- `control/loop_do_while/runs-at-least-once.glsl` - Do-while semantics

## Expected Behavior

- No "index out of bounds" errors during verification
- All do-while loop tests pass
- Variables modified in body are correctly accessible in condition
- Proper phi nodes created for loop variables

## Verification

Run all do-while tests:

```bash
scripts/glsl-filetests.sh control/loop_do_while/
```

Expected result: All tests pass, no verification errors.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix do-while block sealing order and SSA issues"
```

## Notes

- This is a critical fix - do-while loops are completely broken without this
- The error suggests SSA is creating invalid value references
- Block sealing order is crucial for correct SSA construction
- May need to investigate Cranelift's SSA implementation to understand the exact issue
- Consider adding debug logging to see what values are being created/referenced

## Investigation Commands

To debug the issue:

```bash
# Run with backtrace to see exact error location
RUST_BACKTRACE=full scripts/glsl-filetests.sh control/loop_do_while/basic.glsl

# Check generated IR (if available)
scripts/glsl-filetests.sh control/loop_do_while/basic.glsl 2>&1 | grep -A 50 "CLIF IR"
```

## References

- Clang's `EmitDoStmt` pattern (see `00-overview.md`)
- Cranelift SSA documentation: `cranelift/frontend/src/ssa.rs`
- Block sealing: `cranelift/frontend/src/frontend.rs` `seal_block()` method

