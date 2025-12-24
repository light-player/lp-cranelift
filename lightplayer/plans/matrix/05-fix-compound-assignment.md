# Fix Compound Assignment Operators

## Problem

Compound assignment operators (`+=`, `-=`, `*=`, `/=`) are not implemented for matrices.

**Current behavior:**
- Compilation error: "only simple assignment (=) supported"

**Affected tests:**
- `matrix/assignment/compound-assignment.glsl:17` - compilation fails

## Root Cause

The assignment handling in `codegen/expr/mod.rs` only supports simple assignment (`=`). Compound assignment operators need to:
1. Read the current value
2. Perform the operation (add, subtract, multiply, divide)
3. Write the result back

## Fix Strategy

1. **Extend assignment handling** in `translate_assignment`:
   - Detect compound assignment operators (`+=`, `-=`, `*=`, `/=`)
   - For matrices, read current matrix values
   - Perform component-wise operation with RHS
   - Write results back

2. **Handle different RHS types**:
   - Matrix + Matrix: component-wise operation
   - Matrix + Scalar: component-wise operation with scalar

3. **Reuse existing code**:
   - Use `read_lvalue` to get current values
   - Use matrix arithmetic operations from `expr/matrix.rs`
   - Use `write_lvalue` to store results

## Implementation Steps

1. Update `translate_assignment` to handle compound operators
2. Add helper function to perform compound assignment for matrices
3. Ensure type checking allows compound assignment (matrix + matrix, matrix + scalar)
4. Test with all compound operators and matrix sizes

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/mod.rs` - `translate_assignment`
- Possibly add helper in `codegen/expr/matrix.rs` for compound operations

## Test Cases

- `matrix/assignment/compound-assignment.glsl` - all tests should pass
- Test `+=`, `-=`, `*=`, `/=` for mat2, mat3, mat4
- Test with both matrix and scalar RHS


