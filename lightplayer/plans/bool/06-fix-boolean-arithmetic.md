# Phase 6: Fix Boolean Values in Arithmetic

## Goal

Ensure boolean values are properly converted when used in arithmetic operations, fixing type mismatch errors.

## Problem

From nested loop test error, boolean values (i8) are used in arithmetic operations expecting i32:

```
block4:
    v17 = iadd.i32 v11, v16
;   ^~~~~~~~~~~~~~~~~~~~~~~
; error: inst17 (v17 = iadd.i32 v11, v16): arg 1 (v16) has type i8, expected i32
```

**Root Cause**: When boolean values are used in arithmetic expressions, they need explicit conversion to integer types. This happens when boolean expressions are incorrectly used in arithmetic contexts.

**Example**:
```glsl
int sum = 0;
while (i < 2) {
    while (j < 3) {
        sum = sum + i + j;  // If condition result (i8) is used here, type error
        j = j + 1;
    }
    i = i + 1;
}
```

## Solution

Ensure proper type conversion when boolean values are used in arithmetic operations. This may require:

1. **Expression Type Checking**: Ensure expression types match expected types
2. **Type Promotion**: Convert boolean values to integers when used in arithmetic
3. **Fix Expression Evaluation**: Ensure boolean expressions aren't incorrectly used in arithmetic

## Implementation Steps

### Step 1: Identify Problem Locations

Run failing tests to identify where boolean values are incorrectly used:

```bash
scripts/glsl-filetests.sh control/loop_while/nested.glsl
scripts/glsl-filetests.sh bool/ctrl-while.glsl
scripts/glsl-filetests.sh bool/ctrl-for.glsl
scripts/glsl-filetests.sh bool/ctrl-do-while.glsl
```

Look for errors like:
- `arg has type i8, expected i32`
- Boolean values used in arithmetic operations

### Step 2: Fix Expression Type Handling

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`

Ensure that when boolean values are used in arithmetic contexts, they're properly converted:

```rust
// In translate_scalar_binary, for arithmetic operators:
} else {
    // Arithmetic operators: promote to common type
    let common_ty = promote_numeric(lhs_ty, rhs_ty);
    
    // If operands are bool, convert to int first
    let lhs_val = if lhs_ty == &GlslType::Bool {
        // Convert bool to int for arithmetic
        coercion::coerce_to_type(ctx, lhs_val, lhs_ty, &GlslType::Int)?
    } else {
        coercion::coerce_to_type(ctx, lhs_val, lhs_ty, &common_ty)?
    };
    
    let rhs_val = if rhs_ty == &GlslType::Bool {
        // Convert bool to int for arithmetic
        coercion::coerce_to_type(ctx, rhs_val, rhs_ty, &GlslType::Int)?
    } else {
        coercion::coerce_to_type(ctx, rhs_val, rhs_ty, &common_ty)?
    };
    
    // Update common_ty if we converted bools
    let common_ty = if lhs_ty == &GlslType::Bool || rhs_ty == &GlslType::Bool {
        GlslType::Int  // Result is int if bools were involved
    } else {
        common_ty
    };
    
    (lhs_val, rhs_val, common_ty)
}
```

**Note**: This may not be the right approach. The issue might be that boolean expressions shouldn't be used in arithmetic at all. Need to check GLSL spec.

### Step 3: Check GLSL Spec Behavior

According to GLSL spec, boolean values cannot be used directly in arithmetic. They must be explicitly converted:

```glsl
int x = 5;
bool b = true;
int y = x + int(b);  // OK: explicit conversion
int z = x + b;       // ERROR: implicit conversion not allowed
```

So the fix might be to **reject** boolean operands in arithmetic operations at the type checking stage, rather than converting them.

### Step 4: Update Type Checking

**File**: `lightplayer/crates/lp-glsl/src/semantic/type_check/operators.rs`

Ensure arithmetic operators reject boolean operands:

```rust
// Arithmetic operators
Add | Sub | Mult | Div => {
    // ... existing vector/matrix checks ...
    
    // Scalar operations
    if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
        return Err(GlslError::new(
            ErrorCode::E0106,
            format!("arithmetic operator {:?} requires numeric operands", op),
        )
        .with_location(source_span_to_location(&span))
        .with_note(format!(
            "left operand has type `{:?}`, right operand has type `{:?}`",
            lhs_ty, rhs_ty
        )));
    }
    // ... rest ...
}
```

This should already be in place, so the issue might be elsewhere.

### Step 5: Fix Actual Problem

The actual problem from the error is that a boolean condition result (v16, i8) is being used in arithmetic. This suggests the issue is in how loop conditions or boolean expressions are being evaluated.

Looking at the error:
```
block3(v10: i32, v11: i32):
    v16 = select v13, v14, v15  ; v14 = 1, v15 = 0  (i8 boolean)
    brif v16, block4, block5

block4:
    v17 = iadd.i32 v11, v16  ; ERROR: v16 is i8, expected i32
```

This suggests that `v16` (the boolean condition) is being incorrectly used in an arithmetic operation. This might be a bug in the codegen where a boolean value is being reused incorrectly.

**Solution**: Ensure that boolean values from conditions are not reused in arithmetic contexts. The issue might be in variable/expression handling.

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Fix type handling
- `lightplayer/crates/lp-glsl/src/semantic/type_check/operators.rs` - Ensure proper type checking
- Possibly expression evaluation code if boolean values are being incorrectly reused

## Test Cases

All control flow tests with boolean conditions should pass:
- `bool/ctrl-while.glsl` - While loops with boolean conditions
- `bool/ctrl-for.glsl` - For loops with boolean conditions
- `bool/ctrl-do-while.glsl` - Do-while loops with boolean conditions
- `bool/ctrl-ternary.glsl` - Ternary operators with boolean conditions
- `control/loop_while/nested.glsl` - Nested loops (from terminal output)

## Expected Behavior

- Boolean conditions work correctly in control flow
- Boolean values are not incorrectly used in arithmetic
- Type errors are resolved

## Verification

Run control flow tests:

```bash
scripts/glsl-filetests.sh bool/ctrl-while.glsl
scripts/glsl-filetests.sh bool/ctrl-for.glsl
scripts/glsl-filetests.sh bool/ctrl-do-while.glsl
scripts/glsl-filetests.sh control/loop_while/nested.glsl
```

Expected result: All control flow tests pass, no type mismatch errors.

## Commit Instructions

Once tests pass:

```bash
git add -A
git commit -m "lpc: fix boolean values in arithmetic operations"
```

## Notes

- **Root Cause**: Need to identify exactly where boolean values are being incorrectly used in arithmetic
- **GLSL Spec**: Boolean values cannot be used directly in arithmetic - must be explicitly converted
- **Debugging**: May need to add debug output to identify the exact location of the issue
- **Type Safety**: Ensure type checking properly rejects boolean operands in arithmetic





