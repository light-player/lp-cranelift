# Phase 2: Implement Logical OR Operator

## Goal

Implement the `||` (logical OR) operator with proper boolean handling.

## Problem

Logical OR operator returns "not yet implemented" error:

```rust
// In codegen/expr/binary.rs
Or | Xor => {
    return Err(GlslError::new(
        ErrorCode::E0400,
        format!("logical operator {:?} not yet implemented", op),
    ));
}
```

**Error**: `logical operator Or not yet implemented`

**GLSL Spec**: Logical OR (`||`) requires both operands to be `bool`, result is `bool`. Short-circuit evaluation: if left operand is `true`, right operand is not evaluated.

## Solution

Implement logical OR following the same pattern as logical AND, but with OR logic instead of AND.

**Note**: Short-circuit evaluation may require control flow changes, but for now we'll implement the basic version that evaluates both operands. Short-circuit can be added later if needed.

## Implementation Steps

### Step 1: Implement Logical OR Codegen

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`

Update `translate_scalar_binary_op` to implement logical OR:

```rust
// Logical operators (bool only, already validated by type_check)
And => {
    // Logical AND: both operands must be bool (I8)
    // Result: (lhs != 0) && (rhs != 0) ? 1 : 0
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let lhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, lhs, zero);
    let rhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, rhs, zero);
    // Result is 1 if both are non-zero, 0 otherwise
    let rhs_result = ctx.builder.ins().select(rhs_nonzero, one, zero);
    ctx.builder.ins().select(lhs_nonzero, rhs_result, zero)
}
Or => {
    // Logical OR: both operands must be bool (I8)
    // Result: (lhs != 0) || (rhs != 0) ? 1 : 0
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let lhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, lhs, zero);
    let rhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, rhs, zero);
    // Result is 1 if either is non-zero, 0 otherwise
    // Equivalent to: lhs != 0 ? 1 : (rhs != 0 ? 1 : 0)
    let rhs_result = ctx.builder.ins().select(rhs_nonzero, one, zero);
    ctx.builder.ins().select(lhs_nonzero, one, rhs_result)
}
Xor => {
    return Err(GlslError::new(
        ErrorCode::E0400,
        format!("logical operator {:?} not yet implemented", op),
    ));
}
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Implement logical OR

## Test Cases

All logical OR tests should pass:

- `bool/op-or.glsl` - All test cases
- `bool/edge-short-circuit-or.glsl` - May require short-circuit evaluation (can be Phase 6)

## Expected Behavior

- `true || true` → `true`
- `true || false` → `true`
- `false || true` → `true`
- `false || false` → `false`
- `bool a = true; bool b = false; a || b` → `true`
- `(false || false) || true` → `true`

## Verification

Run logical OR tests:

```bash
scripts/glsl-filetests.sh bool/op-or.glsl
scripts/glsl-filetests.sh bool/edge-short-circuit-or.glsl
```

Expected result: `op-or.glsl` passes. `edge-short-circuit-or.glsl` may still fail if short-circuit evaluation is required (can be addressed in Phase 6).

## Commit Instructions

Once tests pass:

```bash
git add -A
git commit -m "lpc: implement logical OR operator"
```

## Notes

- **Short-circuit evaluation**: The current implementation evaluates both operands. True short-circuit evaluation would require control flow changes (evaluating right operand only if left is false). This can be added later if needed.
- **Pattern**: Follows the same pattern as logical AND, but with OR logic (result is 1 if either operand is non-zero).




