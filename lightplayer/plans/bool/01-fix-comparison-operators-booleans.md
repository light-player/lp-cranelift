# Phase 1: Fix Comparison Operators for Booleans

## Goal

Allow `==` and `!=` operators to work on boolean operands, fixing the "comparison operator requires numeric operands" error.

## Problem

The type inference function rejects boolean operands for comparison operators:

```rust
// In semantic/type_check/operators.rs
Equal | NonEqual | LT | GT | LTE | GTE => {
    if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
        return Err(...);  // Rejects Bool operands
    }
    Ok(Type::Bool)
}
```

**Error**: `error[E0106]: comparison operator Equal requires numeric operands`

**GLSL Spec**: Comparison operators `==` and `!=` work on all types including booleans. The result is always `bool`.

## Solution

Update type inference and codegen to support boolean comparisons:

1. **Type Inference**: Allow boolean operands for `==` and `!=` operators
2. **Codegen**: Implement boolean comparison logic in `translate_scalar_binary_op`

## Implementation Steps

### Step 1: Update Type Inference

**File**: `lightplayer/crates/lp-glsl/src/semantic/type_check/operators.rs`

Update `infer_binary_result_type` to allow boolean operands for equality operators:

```rust
// Comparison operators: operands must be compatible, result is bool
Equal | NonEqual => {
    // Equality operators work on all types (including bool)
    if lhs_ty != rhs_ty {
        return Err(GlslError::new(
            ErrorCode::E0106,
            format!("equality operator {:?} requires matching types", op),
        )
        .with_location(source_span_to_location(&span))
        .with_note(format!(
            "left operand has type `{:?}`, right operand has type `{:?}`",
            lhs_ty, rhs_ty
        )));
    }
    Ok(Type::Bool)
}
LT | GT | LTE | GTE => {
    // Relational operators require numeric operands
    if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
        return Err(GlslError::new(
            ErrorCode::E0106,
            format!("comparison operator {:?} requires numeric operands", op),
        )
        .with_location(source_span_to_location(&span))
        .with_note(format!(
            "left operand has type `{:?}`, right operand has type `{:?}`",
            lhs_ty, rhs_ty
        )));
    }
    Ok(Type::Bool)
}
```

### Step 2: Update Binary Operation Handling

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`

Update `translate_scalar_binary` to handle boolean comparisons:

```rust
let (lhs_val, rhs_val, operand_ty) = if is_logical {
    // Logical operators: both operands must be Bool (validated above)
    (lhs_val, rhs_val, GlslType::Bool)
} else if is_comparison {
    // Comparison operators: handle boolean and numeric separately
    if matches!(op, Equal | NonEqual) && lhs_ty == &GlslType::Bool {
        // Boolean equality: no promotion needed
        (lhs_val, rhs_val, GlslType::Bool)
    } else {
        // Numeric comparison: may need promotion
        let common_ty = promote_numeric(lhs_ty, rhs_ty);
        let lhs_val = coercion::coerce_to_type(ctx, lhs_val, lhs_ty, &common_ty)?;
        let rhs_val = coercion::coerce_to_type(ctx, rhs_val, rhs_ty, &common_ty)?;
        (lhs_val, rhs_val, common_ty)
    }
} else {
    // Arithmetic operators: promote to common type
    let common_ty = promote_numeric(lhs_ty, rhs_ty);
    let lhs_val = coercion::coerce_to_type(ctx, lhs_val, lhs_ty, &common_ty)?;
    let rhs_val = coercion::coerce_to_type(ctx, rhs_val, rhs_ty, &common_ty)?;
    (lhs_val, rhs_val, common_ty)
};
```

### Step 3: Implement Boolean Comparison Codegen

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`

Update `translate_scalar_binary_op` to handle boolean comparisons:

```rust
Equal => {
    let cmp_result = match operand_ty {
        GlslType::Bool => {
            // Boolean equality: compare directly as i8
            ctx.builder.ins().icmp(IntCC::Equal, lhs, rhs)
        }
        GlslType::Int => ctx.builder.ins().icmp(IntCC::Equal, lhs, rhs),
        GlslType::Float => ctx.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
        _ => {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!("equal not supported for {:?}", operand_ty),
            ));
        }
    };
    // Convert I1 to I8: select 1 if true, 0 if false
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    ctx.builder.ins().select(cmp_result, one, zero)
}
NonEqual => {
    let cmp_result = match operand_ty {
        GlslType::Bool => {
            // Boolean inequality: compare directly as i8
            ctx.builder.ins().icmp(IntCC::NotEqual, lhs, rhs)
        }
        GlslType::Int => ctx.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
        GlslType::Float => ctx.builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs),
        _ => {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!("nonEqual not supported for {:?}", operand_ty),
            ));
        }
    };
    let one = ctx.builder.ins().iconst(types::I8, 1);
    let zero = ctx.builder.ins().iconst(types::I8, 0);
    ctx.builder.ins().select(cmp_result, one, zero)
}
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/semantic/type_check/operators.rs` - Update type inference
- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Update codegen

## Test Cases

All equality tests should pass:

- `bool/op-equal.glsl` - All test cases
- `bool/op-not-equal.glsl` - All test cases

## Expected Behavior

- `true == true` → `true`
- `true == false` → `false`
- `false == false` → `true`
- `bool a = true; bool b = false; a == b` → `false`
- `bool a = true; a == a` → `true`

## Verification

Run boolean equality tests:

```bash
scripts/glsl-filetests.sh bool/op-equal.glsl
scripts/glsl-filetests.sh bool/op-not-equal.glsl
```

Expected result: All equality tests pass.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix comparison operators to support boolean operands"
```



