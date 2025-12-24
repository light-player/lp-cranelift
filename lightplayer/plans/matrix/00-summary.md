# Matrix Implementation Fixes - Summary

This directory contains plans for fixing matrix-related issues in the GLSL compiler.

## NOTES

- Matrix access is column-major: mat[col][row]
- GLSL Spec is here: /Users/yona/dev/photomancer/glsl-spec/chapters

Many tests have wrong expecations and comments around matrix storage order that assume row-major. These should be updated to use column-major.

## Issues Identified

1. **01-fix-matrix-column-access.md** - `m[col]` returns rows instead of columns
2. **02-fix-matrix-transpose.md** - Transpose has incorrect index calculation
3. **03-fix-outer-product.md** - Outer product uses wrong formula/dimensions
4. **04-fix-matrix-element-indexing.md** - Matrix element access and arithmetic operations producing wrong results
5. **05-fix-compound-assignment.md** - Compound assignment operators (`+=`, `-=`, etc.) not implemented
6. **06-implement-4x4-determinant.md** - 4x4 determinant not implemented
7. **07-implement-matrix-inverse.md** - 3x3 and 4x4 inverse not implemented
8. **08-implement-matrix-constructors.md** - Matrix-to-matrix and mixed constructors not implemented

## Priority Order

**Critical (affects correctness):**

1. Fix matrix column access (01) - affects many operations
2. Fix matrix transpose (02) - affects matrix operations
3. Fix outer product (03) - affects builtin function
4. Fix matrix element indexing (04) - affects all matrix operations

**Missing Features:** 5. Fix compound assignment (05) - compilation error 6. Implement 4x4 determinant (06) - needed for 4x4 inverse 7. Implement matrix inverse (07) - depends on 06 8. Implement matrix constructors (08) - compilation errors

## Test Status

Before fixes: 47 passed, 17 failed
After fixes: All matrix tests should pass

## Dependencies

- Plan 06 (4x4 determinant) must be completed before Plan 07 (4x4 inverse)
- Plans 01-04 should be done first as they affect correctness of other operations
