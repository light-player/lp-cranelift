# Implement Missing Matrix Constructors

## Problem

Matrix-to-matrix constructors and mixed scalar/vector constructors are not implemented.

**Current behavior:**
- Compilation error: "mat3 constructor has wrong number of arguments" (for matrix-to-matrix)
- Compilation error: "matrix column 0 must be a vector, got Float" (for mixed constructors)

**Affected tests:**
- `matrix/constructors/from-matrix.glsl:18` - compilation fails
- `matrix/constructors/mixed.glsl:22` - compilation fails

## Root Cause

The matrix constructor type checking and code generation in `semantic/type_check/constructors.rs` and `codegen/expr/constructor.rs` don't handle:
1. Matrix-to-matrix constructors (e.g., `mat3(mat2)`)
2. Mixed scalar/vector constructors (e.g., `mat2(vec2(1,2), 3.0, 4.0)`)

## Implementation Strategy

### Matrix-to-Matrix Constructors

When constructing a larger matrix from a smaller one:
- Copy the smaller matrix into the top-left corner
- Fill remaining elements:
  - Diagonal elements: 1.0 (identity)
  - Off-diagonal elements: 0.0

Examples:
- `mat3(mat2 m)`: Copy m into top-left 2x2, set `[2][2] = 1.0`, rest 0.0
- `mat4(mat2 m)`: Copy m into top-left 2x2, set `[2][2] = 1.0, [3][3] = 1.0`, rest 0.0
- `mat4(mat3 m)`: Copy m into top-left 3x3, set `[3][3] = 1.0`, rest 0.0

### Mixed Scalar/Vector Constructors

Allow mixing vectors and scalars:
- `mat2(vec2(1,2), 3.0, 4.0)`: First column from vector, remaining from scalars
- `mat3(vec3(1,2,3), vec3(4,5,6), 7.0, 8.0, 9.0)`: First two columns from vectors, last column from scalars

Rules:
- Arguments fill matrix in column-major order
- Vectors contribute their elements as a column
- Scalars contribute single elements
- Total elements must match matrix size

## Implementation Steps

1. **Update type checking** (`semantic/type_check/constructors.rs`):
   - Add case for matrix argument in `check_matrix_constructor`
   - Add case for mixed scalar/vector arguments
   - Validate total element count matches matrix size

2. **Update code generation** (`codegen/expr/constructor.rs`):
   - Handle matrix-to-matrix: extract elements from source matrix, pad with identity
   - Handle mixed arguments: process vectors and scalars in order, fill column-major

## Files to Modify

- `lightplayer/crates/lp-glsl/src/semantic/type_check/constructors.rs` - `check_matrix_constructor`
- `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs` - `translate_matrix_constructor`

## Test Cases

- `matrix/constructors/from-matrix.glsl` - all tests should pass
- `matrix/constructors/mixed.glsl` - all tests should pass
- Test mat2→mat3, mat2→mat4, mat3→mat4
- Test various mixed constructor patterns


