# Implement 4x4 Matrix Determinant

## Problem

4x4 matrix determinant is not implemented.

**Current behavior:**
- Compilation error: "4x4 determinant not yet implemented"

**Affected tests:**
- `matrix/builtins/determinant.glsl:65` - compilation fails
- `matrix/edge-cases/singular-matrix.glsl:53` - compilation fails (4x4 singular matrix)

## Root Cause

The `builtin_determinant` function in `builtins/matrix.rs` only implements 2x2 and 3x3 determinants. 4x4 determinant needs to be added.

## Implementation Strategy

Use cofactor expansion (Laplace expansion) along the first row:
```
det(M) = Σ(-1)^(1+j) * M[0][j] * det(M_minor_j)
```

Where `M_minor_j` is the 3x3 matrix obtained by removing row 0 and column j.

Steps:
1. For each column j in the first row
2. Compute the sign: `(-1)^(1+j)`
3. Extract the 3x3 minor matrix (remove row 0, column j)
4. Recursively compute the 3x3 determinant (already implemented)
5. Sum: `sign * M[0][j] * det(minor)`

## Implementation Details

1. **Extract minor matrix**: Helper function to extract 3x3 minor from 4x4 matrix
2. **Compute sign**: `(-1)^(1+j)` = `1` if j is even, `-1` if j is odd
3. **Recursive call**: Use existing 3x3 determinant implementation
4. **Sum results**: Accumulate the cofactor expansion

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/builtins/matrix.rs` - `builtin_determinant`

## Test Cases

- `matrix/builtins/determinant.glsl` - 4x4 test should pass
- `matrix/edge-cases/singular-matrix.glsl` - 4x4 singular matrix (det = 0)
- Test with identity matrix (det = 1)
- Test with known 4x4 matrices

## Notes

- Consider numerical stability for large matrices
- May want to use a more stable algorithm (like LU decomposition) in the future, but cofactor expansion is fine for now


