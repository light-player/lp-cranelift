# Phase 4: Fix Type Handling in Control Flow

## Goal

Fix type handling issues where boolean values (i8) are incorrectly used as integers (i32) in control flow paths, causing type mismatch verifier errors.

## Problem

Boolean comparison results (i8) are incorrectly used as integers (i32) in arithmetic operations, causing type mismatch errors.

**Example Error**:
```
block5:
    v25 = iadd.i32 v19, v23  ; v23 = 0
;   ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
; error: inst25 (v25 = iadd.i32 v19, v23  ; v23 = 0): arg 1 (v23) has type i8, expected i32
```

**Root Cause**: The value `v23` is an i8 (boolean result from `select`), but it's being used as an i32 in an addition operation.

## Root Cause Analysis

### Issue: Boolean Values in Expressions

When boolean values are used in arithmetic expressions, they need to be converted to integers. The current codegen doesn't always perform this conversion.

**Example**:
```glsl
int test_complex_nested_1() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        if (i % 2 == 0) {
            for (int j = 0; j < 2; j++) {
                sum = sum + i + j;
            }
        }
    }
    return sum;
}
```

The condition `i % 2 == 0` produces a boolean (i8), but if this value is somehow used in arithmetic, it causes a type error.

## Solution

### Solution: Proper Type Conversion

When boolean values are used in contexts requiring integers, convert them explicitly:

1. **Boolean to Integer Conversion**: Use `uextend` or `sextend` to convert i8 to i32
2. **Condition Evaluation**: Ensure conditions are properly evaluated as booleans
3. **Expression Type Checking**: Ensure expression types match expected types

**Pattern**:
```rust
// Boolean comparison result (i8)
let cond_value: Value = ctx.builder.ins().icmp_eq(lhs, rhs);  // Returns i8

// If used in arithmetic, convert to i32
let cond_i32: Value = ctx.builder.ins().uextend(I32, cond_value);  // i8 -> i32
let result: Value = ctx.builder.ins().iadd(operand, cond_i32);
```

## Implementation Steps

### Step 1: Identify Type Mismatch Locations

1. Run failing test to see exact error:

   ```bash
   scripts/glsl-filetests.sh control/nested/complex.glsl:23
   ```

2. Check where boolean values are incorrectly used:
   - Look for `select` instructions producing i8 values
   - Check if these values are used in arithmetic operations
   - Identify where type conversion is needed

### Step 2: Fix Type Conversion

1. Update expression evaluation to handle boolean-to-integer conversion:

   - Check `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs`
   - Ensure boolean values are converted when used in arithmetic
   - Add type conversion helpers if needed

2. Fix control flow condition handling:

   - Check `lightplayer/crates/lp-glsl/src/codegen/stmt/loops.rs` - `translate_condition`
   - Ensure conditions are properly typed as booleans
   - Ensure boolean values aren't incorrectly used in expressions

### Step 3: Test Type Handling

Run nested control flow tests:

```bash
scripts/glsl-filetests.sh control/nested/complex.glsl
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/binary.rs` - Fix boolean-to-integer conversion
- `lightplayer/crates/lp-glsl/src/codegen/stmt/loops.rs` - Ensure proper condition typing
- `lightplayer/crates/lp-glsl/src/codegen/context.rs` - Add type conversion helpers if needed

## Test Cases

- `control/nested/complex.glsl` - Nested control flow with type issues
- `control/edge_cases/condition-expressions.glsl` - Condition expression tests (some may pass, some may fail due to missing OR operator)

## Expected Behavior

- Boolean values are properly converted when used in integer contexts
- No type mismatch verifier errors
- Conditions are properly typed as booleans
- Expressions have correct types

## Verification

Run all control flow tests:

```bash
scripts/glsl-filetests.sh control/
```

Expected result: Type handling tests pass, no type mismatch errors.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: fix type handling in control flow (boolean to integer conversion)"
```

## Notes

- This is a type safety issue - boolean values (i8) must be converted to integers (i32) when used in arithmetic
- Cranelift's type system is strict - type mismatches cause verifier errors
- The fix may require adding explicit type conversion instructions (`uextend` or `sextend`)
- This fix can be done independently of other phases

