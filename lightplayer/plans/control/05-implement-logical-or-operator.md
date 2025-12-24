# Phase 5: Implement Logical OR Operator

## Goal

Implement the missing logical OR operator (`||`) to support complex condition expressions in control flow.

## Problem

The logical OR operator (`||`) is not yet implemented, causing compilation errors:

**Error**: `error[E0400]: logical operator Or not yet implemented`

**Affected Tests**:
- `control/edge_cases/condition-expressions.glsl:98` - `test_if_logical_or()`
- `control/loop_while/complex-condition.glsl:33` - `test_while_loop_complex_condition_or()`
- `control/loop_for/complex-condition.glsl:30` - `test_for_loop_complex_condition_or()`

## Root Cause

The binary expression handler doesn't handle the `Or` operator case. Looking at the code, logical AND (`&&`) may be implemented, but OR (`||`) is missing.

## Solution

### Solution: Implement Logical OR Operator

Logical OR should implement short-circuit evaluation:
- Evaluate left operand
- If true, result is true (don't evaluate right operand)
- If false, evaluate right operand and use its result

**Pattern** (similar to logical AND):
```rust
Expr::Binary(BinaryOp::Or, left, right) => {
    // Short-circuit evaluation
    let left_val = self.emit_rvalue(left)?;
    let left_bool = self.convert_to_bool(left_val)?;
    
    // Create blocks for short-circuit
    let right_block = self.builder.create_block();
    let merge_block = self.builder.create_block();
    
    // Branch: if left is true, go to merge with true, else evaluate right
    self.emit_cond_branch(left_bool, merge_block, right_block)?;
    
    // Right operand evaluation
    self.emit_block(right_block);
    let right_val = self.emit_rvalue(right)?;
    let right_bool = self.convert_to_bool(right_val)?;
    self.emit_branch(merge_block)?;
    
    // Merge block: phi node for result
    self.emit_block(merge_block);
    // Create phi: true from left branch, right_bool from right branch
    let result = self.builder.ins().select(left_bool, 
        self.builder.ins().iconst(I8, 1),  // true if left was true
        right_bool);  // right_bool if left was false
    
    Ok(RValue::Scalar(result))
}
```

## Implementation Steps

### Step 1: Check Existing Logical AND Implementation

1. Check how logical AND is implemented:

   ```bash
   grep -r "And\|&&" lightplayer/crates/lp-glsl/src/codegen/expr/
   ```

2. Use logical AND as a reference for implementing OR

### Step 2: Implement Logical OR Operator

1. Update `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`:

   - Add case for `BinaryOp::Or`
   - Implement short-circuit evaluation
   - Ensure proper block management
   - Ensure proper phi node creation

2. Key requirements:
   - Short-circuit evaluation (don't evaluate right if left is true)
   - Proper block creation and branching
   - Proper phi node for result value
   - Return boolean (i8) result

### Step 3: Test Logical OR

Run tests that use logical OR:

```bash
scripts/glsl-filetests.sh control/edge_cases/condition-expressions.glsl:98
scripts/glsl-filetests.sh control/loop_while/complex-condition.glsl:33
scripts/glsl-filetests.sh control/loop_for/complex-condition.glsl:30
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Add logical OR operator handling
- Possibly `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - Ensure OR is dispatched correctly

## Test Cases

- `control/edge_cases/condition-expressions.glsl:98` - `test_if_logical_or()`
- `control/loop_while/complex-condition.glsl:33` - `test_while_loop_complex_condition_or()`
- `control/loop_for/complex-condition.glsl:30` - `test_for_loop_complex_condition_or()`

## Expected Behavior

- Logical OR operator works correctly
- Short-circuit evaluation: right operand not evaluated if left is true
- Proper boolean result (i8)
- Works in if conditions, while conditions, for conditions

## Verification

Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

Expected result: Logical OR tests pass, complex condition expressions work.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement logical OR operator (||)"
```

## Notes

- This is a separate feature implementation, not a bug fix
- Logical OR should follow the same pattern as logical AND
- Short-circuit evaluation is important for correctness and performance
- This fix can be done independently of other phases

## Reference: Logical AND Pattern

If logical AND is already implemented, use it as a reference:

```rust
// Logical AND pattern (for reference)
Expr::Binary(BinaryOp::And, left, right) => {
    // Short-circuit: if left is false, result is false
    let left_val = self.emit_rvalue(left)?;
    let left_bool = self.convert_to_bool(left_val)?;
    
    let right_block = self.builder.create_block();
    let merge_block = self.builder.create_block();
    
    // Branch: if left is false, go to merge with false, else evaluate right
    self.emit_cond_branch(left_bool, right_block, merge_block)?;
    
    // Right operand evaluation
    self.emit_block(right_block);
    let right_val = self.emit_rvalue(right)?;
    let right_bool = self.convert_to_bool(right_val)?;
    self.emit_branch(merge_block)?;
    
    // Merge block: phi node
    self.emit_block(merge_block);
    let result = self.builder.ins().select(left_bool, 
        right_bool,  // right_bool if left was true
        self.builder.ins().iconst(I8, 0));  // false if left was false
    
    Ok(RValue::Scalar(result))
}
```

