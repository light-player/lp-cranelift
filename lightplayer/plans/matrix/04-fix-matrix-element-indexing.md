# Fix Matrix Element Indexing

## Problem

Matrix element access and related operations are producing incorrect results. This affects:
- Matrix arithmetic operations (mat4 getting mat2 results)
- Matrix constructors
- Matrix assignment
- Element increment/decrement

**Current behavior:**
- `test_mat4_addition`: expected `308.0`, got `110.0` (mat2 result)
- `test_mat4_subtraction`: expected `252.0`, got `90.0` (mat2 result)
- `test_mat4_division`: expected `140.0`, got `50.0` (mat2 result)
- `test_mat4_negation`: expected `-28.0`, got `-10.0` (mat2 result)
- `test_mat4_assignment`: expected `280.0`, got `100.0` (mat2 result)
- `test_mat4_from_scalars`: expected `28.0`, got `10.0` (row 0 instead of column 0)
- `test_mat4_precision`: expected `0.0028`, got `0.0010070801`

**Affected tests:**
- Multiple mat4 operations returning mat2-sized results
- Matrix constructors producing wrong element ordering

## Root Cause

The issue appears to be related to:
1. **Matrix size confusion**: mat4 operations returning mat2 results suggests the wrong number of elements are being processed
2. **Index calculation**: Element access `m[col][row]` may be using wrong indices
3. **Storage order**: Matrix construction or element access may not match column-major storage

The fact that mat4 tests are getting mat2 results (exactly 4 elements instead of 16) suggests:
- The test runner might be calling the wrong function, OR
- The matrix operations are only processing the first 4 elements (treating mat4 as mat2)

## Investigation Steps

1. **Check matrix arithmetic operations** (`codegen/expr/matrix.rs`):
   - Verify mat4 addition/subtraction processes all 16 elements
   - Check if there's a size check that's incorrectly limiting to mat2

2. **Check matrix constructors** (`codegen/expr/constructor.rs`):
   - Verify mat4 constructor stores all 16 elements correctly
   - Check column-major ordering: `[col0_row0, col0_row1, ..., col1_row0, ...]`

3. **Check element access** (`codegen/expr/component.rs`, `codegen/lvalue.rs`):
   - Verify `m[col][row]` uses correct index: `col * rows + row`
   - Check if matrix size is being correctly determined

4. **Check test runner**:
   - Verify the correct test function is being called
   - Check if there's a function name collision or selection issue

## Fix Strategy

1. **Verify matrix size handling**:
   - Ensure mat4 operations process all 16 elements
   - Check that matrix type information is correctly propagated

2. **Fix element indexing**:
   - Verify `m[col][row]` uses `col * rows + row` for column-major storage
   - Ensure constructors fill matrices in column-major order

3. **Fix arithmetic operations**:
   - Ensure component-wise operations process all elements
   - Verify result matrix has correct size

4. **Fix constructors**:
   - Verify scalar constructor fills in column-major order
   - Check that all elements are stored correctly

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/matrix.rs` - arithmetic operations
- `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs` - matrix constructors
- `lightplayer/crates/lp-glsl/src/codegen/expr/component.rs` - element access
- `lightplayer/crates/lp-glsl/src/codegen/lvalue.rs` - LValue resolution

## Test Cases

- `matrix/arithmetic/addition.glsl` - mat4 test
- `matrix/arithmetic/subtraction.glsl` - mat4 test
- `matrix/arithmetic/division.glsl` - mat4 test
- `matrix/arithmetic/negation.glsl` - mat4 test
- `matrix/assignment/simple-assignment.glsl` - mat4 test
- `matrix/constructors/from-scalars.glsl` - mat4 test
- `matrix/edge-cases/large-values.glsl` - mat4 precision test


