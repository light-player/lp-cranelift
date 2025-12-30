# Boolean Operations Fixes - Overview

This directory contains plans for fixing boolean operation compilation issues. These fixes are prerequisites for loop tests that use boolean conditions and expressions.

## Problem Statement

The current boolean operation implementation has several critical issues:

1. **Comparison Operators on Booleans**: `==` and `!=` operators don't work on boolean operands, even though GLSL allows comparing booleans
2. **Missing Logical Operators**: Logical OR (`||`) and XOR (`^^`) are not implemented
3. **Missing Scalar Type Constructors**: Constructors like `bool(int)`, `int(bool)`, `float(bool)`, `uint(bool)` are undefined
4. **Incomplete Type Conversions**: Coercion module only handles `int → float`, missing all boolean conversions
5. **Type Mismatch Errors**: Boolean values (i8) incorrectly used as integers (i32) in arithmetic operations, causing verifier errors

## Current Test Status

**20 failing tests** out of 23 boolean tests:

### Passing (3):
- `bool/ctrl-if.glsl` - If statements with boolean conditions
- `bool/op-and.glsl` - Logical AND operator
- `bool/op-not.glsl` - Logical NOT operator

### Failing (20):

**Assignment & Basic Operations:**
- `bool/assign-simple.glsl` - Fails on `bool(5)` constructor (undefined function)
- `bool/op-equal.glsl` - Comparison requires numeric operands error
- `bool/op-not-equal.glsl` - Comparison requires numeric operands error
- `bool/op-or.glsl` - Logical OR not implemented
- `bool/op-xor.glsl` - Logical XOR not implemented

**Control Flow:**
- `bool/ctrl-do-while.glsl` - Likely boolean condition/type issues
- `bool/ctrl-for.glsl` - Likely boolean condition/type issues
- `bool/ctrl-ternary.glsl` - Likely boolean condition/type issues
- `bool/ctrl-while.glsl` - Likely boolean condition/type issues

**Type Conversions:**
- `bool/from-bool.glsl` - Constructor/conversion issues
- `bool/from-float.glsl` - Constructor/conversion issues
- `bool/from-int.glsl` - Constructor/conversion issues
- `bool/from-uint.glsl` - Constructor/conversion issues
- `bool/to-float.glsl` - Constructor/conversion issues
- `bool/to-int.glsl` - Constructor/conversion issues
- `bool/to-uint.glsl` - Constructor/conversion issues

**Edge Cases:**
- `bool/edge-literals.glsl` - Literal handling issues
- `bool/edge-nested.glsl` - Nested boolean expressions
- `bool/edge-short-circuit-and.glsl` - Short-circuit evaluation
- `bool/edge-short-circuit-or.glsl` - Short-circuit evaluation (requires OR)

## Root Cause Analysis

### Issue 1: Comparison Operators Don't Support Booleans

**Problem**: The type inference function `infer_binary_result_type` in `operators.rs` rejects boolean operands for comparison operators:

```rust
Equal | NonEqual | LT | GT | LTE | GTE => {
    if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
        return Err(...);  // Rejects Bool operands
    }
    Ok(Type::Bool)
}
```

**GLSL Spec**: According to GLSL spec, comparison operators (`==`, `!=`) work on all types including booleans. The result is always `bool`.

**Solution**: Update type inference to allow boolean operands for `==` and `!=` operators.

### Issue 2: Missing Logical Operators

**Problem**: Logical OR (`||`) and XOR (`^^`) return "not yet implemented" errors in `binary.rs`:

```rust
Or | Xor => {
    return Err(GlslError::new(
        ErrorCode::E0400,
        format!("logical operator {:?} not yet implemented", op),
    ));
}
```

**Solution**: Implement logical OR and XOR operators following the same pattern as logical AND.

### Issue 3: Scalar Type Constructors Not Handled

**Problem**: Function call handler in `function.rs` only checks for vector/matrix constructors, not scalar constructors:

```rust
if is_vector_type_name(func_name) {
    return constructor::translate_vector_constructor(...);
}
if is_matrix_type_name(func_name) {
    return constructor::translate_matrix_constructor(...);
}
// No handling for bool(int), int(bool), etc.
```

**Solution**: Add scalar constructor handling for `bool`, `int`, `float`, `uint` type names.

### Issue 4: Missing Type Conversions

**Problem**: The coercion module (`coercion.rs`) only handles `int → float` conversion:

```rust
match (from_ty, to_ty) {
    (GlslType::Int, GlslType::Float) => { ... }
    _ => Err(...)  // All other conversions fail
}
```

**Missing Conversions**:
- `bool → int`: false → 0, true → 1
- `bool → float`: false → 0.0, true → 1.0
- `bool → uint`: false → 0u, true → 1u
- `int → bool`: 0 → false, non-zero → true
- `float → bool`: 0.0 → false, non-zero → true
- `uint → bool`: 0 → false, non-zero → true

**Solution**: Add all boolean conversion cases to the coercion module.

### Issue 5: Boolean Values in Arithmetic

**Problem**: From nested loop test error, boolean values (i8) are used in arithmetic operations expecting i32:

```
v17 = iadd.i32 v11, v16
; error: arg 1 (v16) has type i8, expected i32
```

**Root Cause**: When boolean values are used in arithmetic expressions, they need explicit conversion to integer types. This happens when boolean expressions are incorrectly used in arithmetic contexts.

**Solution**: Ensure proper type conversion when boolean values are used in arithmetic operations. This may require fixing expression evaluation to properly handle type promotion.

## Reference Architecture: GLSL Spec

According to GLSL specification:

### Comparison Operators
- `==` and `!=` work on all types (scalars, vectors, matrices, booleans)
- Result type is always `bool`
- For booleans: `true == true` → `true`, `true == false` → `false`

### Logical Operators
- `&&` (AND): Both operands must be `bool`, result is `bool`
- `||` (OR): Both operands must be `bool`, result is `bool`
- `^^` (XOR): Both operands must be `bool`, result is `bool`
- Short-circuit evaluation: `||` and `&&` evaluate right operand only if needed

### Type Constructors
- `bool(value)`: Converts numeric types to bool (0/0.0 → false, non-zero → true)
- `int(bool)`: Converts bool to int (false → 0, true → 1)
- `float(bool)`: Converts bool to float (false → 0.0, true → 1.0)
- `uint(bool)`: Converts bool to uint (false → 0u, true → 1u)

### Type Conversions
- Implicit conversions: Only `int → float` is allowed implicitly
- Explicit conversions: All conversions via constructors are allowed
- Boolean conversions: Always explicit (via constructors)

## Fix Phases

### Phase 1: Fix Comparison Operators for Booleans

**Goal**: Allow `==` and `!=` operators to work on boolean operands

- Update `infer_binary_result_type` to allow boolean operands for `==` and `!=`
- Update `translate_scalar_binary_op` to handle boolean comparisons
- Test with `bool/op-equal.glsl` and `bool/op-not-equal.glsl`

### Phase 2: Implement Logical OR Operator

**Goal**: Implement `||` operator with short-circuit evaluation

- Implement logical OR in `translate_scalar_binary_op`
- Handle short-circuit evaluation (may require control flow changes)
- Test with `bool/op-or.glsl` and `bool/edge-short-circuit-or.glsl`

### Phase 3: Implement Logical XOR Operator

**Goal**: Implement `^^` operator

- Implement logical XOR in `translate_scalar_binary_op`
- Test with `bool/op-xor.glsl`

### Phase 4: Add Scalar Type Constructors

**Goal**: Support `bool(int)`, `int(bool)`, `float(bool)`, `uint(bool)` constructors

- Add scalar constructor detection in `function.rs`
- Implement scalar constructor translation
- Add type checking for scalar constructors
- Test with conversion tests (`from-*`, `to-*`)

### Phase 5: Add Boolean Type Conversions

**Goal**: Support all boolean conversion cases in coercion module

- Add `bool → int`, `bool → float`, `bool → uint` conversions
- Add `int → bool`, `float → bool`, `uint → bool` conversions
- Update coercion module to handle all cases
- Test with conversion tests

### Phase 6: Fix Boolean Values in Arithmetic

**Goal**: Ensure boolean values are properly converted when used in arithmetic

- Identify where boolean values are incorrectly used in arithmetic
- Add proper type conversion/promotion
- May require fixing expression evaluation
- Test with control flow tests that use boolean expressions

## Test Commands

### Run all boolean tests:
```bash
scripts/glsl-filetests.sh bool/
```

### Run specific test category:
```bash
# Assignment tests
scripts/glsl-filetests.sh bool/assign-simple.glsl

# Operator tests
scripts/glsl-filetests.sh bool/op-equal.glsl
scripts/glsl-filetests.sh bool/op-or.glsl

# Conversion tests
scripts/glsl-filetests.sh bool/from-int.glsl
scripts/glsl-filetests.sh bool/to-int.glsl

# Control flow tests
scripts/glsl-filetests.sh bool/ctrl-while.glsl
```

### Run specific test case:
```bash
scripts/glsl-filetests.sh bool/op-equal.glsl:13
```

## Dependencies

- Phase 1 (comparison operators) - Can be done independently
- Phase 2 (logical OR) - Can be done independently, but required for Phase 6
- Phase 3 (logical XOR) - Can be done independently
- Phase 4 (scalar constructors) - Can be done independently, but required for Phase 5
- Phase 5 (type conversions) - Depends on Phase 4, required for Phase 6
- Phase 6 (arithmetic fixes) - Depends on Phases 2 and 5

## Commit Instructions

After completing each phase:

1. **Verify tests pass:**
   ```bash
   scripts/glsl-filetests.sh bool/
   ```

2. **Commit with appropriate message:**
   ```bash
   git add -A
   git commit -m "lpc: [phase description]"
   ```

3. **Keep commits small and focused** - one logical change per commit

## Notes

- **Follow GLSL spec** - Ensure all implementations match GLSL specification behavior
- **Test frequently** - Run tests after each change to catch regressions early
- **Type safety** - Ensure all type conversions are explicit and correct
- **Short-circuit evaluation** - Logical operators should short-circuit when possible (may require control flow changes)





