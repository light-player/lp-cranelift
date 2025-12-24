# Implement Matrix Inverse

## Problem

3x3 and 4x4 matrix inverse are not implemented.

**Current behavior:**
- Compilation error: "3x3 matrix inverse not yet implemented"
- Compilation error: "4x4 matrix inverse not yet implemented"

**Affected tests:**
- `matrix/builtins/inverse.glsl:49` - compilation fails (3x3)
- 4x4 inverse tests will also fail once 4x4 determinant is implemented

## Root Cause

The `builtin_inverse` function in `builtins/matrix.rs` only implements 2x2 inverse. 3x3 and 4x4 need to be added.

## Implementation Strategy

Use the adjugate method:
```
M^(-1) = (1/det(M)) * adj(M)
```

Where `adj(M)` is the adjugate (transpose of cofactor matrix).

Steps:
1. Compute determinant (already implemented for 2x2, 3x3; 4x4 needs to be done first)
2. If determinant is zero, matrix is singular (return error or handle)
3. Compute cofactor matrix: `C[i][j] = (-1)^(i+j) * det(M_minor_ij)`
4. Transpose cofactor matrix to get adjugate
5. Multiply adjugate by `1/det(M)` to get inverse

## Implementation Details

### 3x3 Inverse

1. Compute determinant (already implemented)
2. For each element `[i][j]`:
   - Extract 2x2 minor (remove row i, column j)
   - Compute 2x2 determinant
   - Apply sign: `(-1)^(i+j)`
   - Store in cofactor matrix
3. Transpose cofactor matrix (swap rows and columns)
4. Multiply each element by `1/det`

### 4x4 Inverse

1. Compute determinant (needs to be implemented first - see plan 06)
2. For each element `[i][j]`:
   - Extract 3x3 minor (remove row i, column j)
   - Compute 3x3 determinant (already implemented)
   - Apply sign: `(-1)^(i+j)`
   - Store in cofactor matrix
3. Transpose cofactor matrix
4. Multiply each element by `1/det`

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/builtins/matrix.rs` - `builtin_inverse`

## Dependencies

- Plan 06 (4x4 determinant) must be completed first for 4x4 inverse

## Test Cases

- `matrix/builtins/inverse.glsl` - 3x3 and 4x4 tests should pass
- Test with identity matrix (inverse = identity)
- Test with known invertible matrices
- Test with singular matrices (should handle gracefully)

## Notes

- Consider numerical stability for near-singular matrices
- May want to add tolerance checks for determinant near zero
- The adjugate method is straightforward but computationally expensive for large matrices


