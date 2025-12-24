# Phase 3: Implement Logical XOR Operator

## Goal

Implement the `^^` (logical XOR) operator with proper boolean handling.

## Problem

Logical XOR operator returns "not yet implemented" error:

```rust
// In codegen/expr/binary.rs
Or | Xor => {
    return Err(GlslError::new(
        ErrorCode::E0400,
        format!("logical operator {:?} not yet implemented", op),
    ));
}
```

**Error**: `logical operator Xor not yet implemented`

**GLSL Spec**: Logical XOR (`^^`) requires both operands to be `bool`, result is `bool`. XOR is true when operands differ.

## Solution

Implement logical XOR following the same pattern as logical AND/OR, but with XOR logic (true when operands differ).

## Implementation Steps

### Step 1: Implement Logical XOR Codegen

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`

Update `translate_scalar_binary_op` to implement logical XOR:

```rust
Or => {
    // Logical OR: both operands must be bool (I8)
    // Result: (lhs != 0) || (rhs != 0) ? 1 : 0
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let lhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, lhs, zero);
    let rhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, rhs, zero);
    let rhs_result = ctx.builder.ins().select(rhs_nonzero, one, zero);
    ctx.builder.ins().select(lhs_nonzero, one, rhs_result)
}
Xor => {
    // Logical XOR: both operands must be bool (I8)
    // Result: (lhs != 0) ^^ (rhs != 0) ? 1 : 0
    // XOR is true when operands differ
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let lhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, lhs, zero);
    let rhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, rhs, zero);
    // XOR: true if lhs != rhs (one is true, other is false)
    // Equivalent to: (lhs != 0) != (rhs != 0)
    let lhs_bool = ctx.builder.ins().select(lhs_nonzero, one, zero);
    let rhs_bool = ctx.builder.ins().select(rhs_nonzero, one, zero);
    let xor_result = ctx.builder.ins().icmp(IntCC::NotEqual, lhs_bool, rhs_bool);
    ctx.builder.ins().select(xor_result, one, zero)
}
```

**Alternative simpler implementation** (if boolean values are already 0/1):

```rust
Xor => {
    // Logical XOR: both operands must be bool (I8)
    // Result: (lhs != 0) ^^ (rhs != 0) ? 1 : 0
    // XOR is true when operands differ
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    let one = ctx.builder.ins().iconst(types::I8, 1);
    // If both are 0 or both are 1, result is 0. Otherwise result is 1.
    // Equivalent to: lhs != rhs
    let xor_result = ctx.builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
    ctx.builder.ins().select(xor_result, one, zero)
}
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Implement logical XOR

## Test Cases

All logical XOR tests should pass:
- `bool/op-xor.glsl` - All test cases

## Expected Behavior

- `true ^^ true` → `false`
- `true ^^ false` → `true`
- `false ^^ true` → `true`
- `false ^^ false` → `false`
- `bool a = true; bool b = false; a ^^ b` → `true`
- `bool a = true; a ^^ a` → `false`

## Verification

Run logical XOR tests:

```bash
scripts/glsl-filetests.sh bool/op-xor.glsl
```

Expected result: All XOR tests pass.

## Commit Instructions

Once tests pass:

```bash
git add -A
git commit -m "lpc: implement logical XOR operator"
```

## Notes

- **XOR Logic**: XOR is true when operands differ (one true, one false)
- **Simpler Implementation**: Since boolean values are stored as 0/1 (i8), we can directly compare them with `icmp NotEqual` to get XOR result
- **Pattern**: Follows the same pattern as logical AND/OR, but with XOR logic

