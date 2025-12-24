# Fix Matrix Column Access

## Problem

Matrix column access `m[col]` is returning row data instead of column data. According to GLSL spec, `m[col]` should return the `col`-th column as a vector.

**Current behavior:**
- `m[0]` returns `[1.0, 2.0, 3.0, 4.0]` (row 0)
- Expected: `[1.0, 5.0, 9.0, 13.0]` (column 0)

**Affected tests:**
- `matrix/indexing/column-access.glsl:73` - expected 28.0, got 10.0
- `matrix/indexing/column-access.glsl:89` - expected 36.0, got 42.0

## Root Cause

The indexing logic in `translate_matrix_indexing` (in `codegen/expr/component.rs`) appears correct:
```rust
let idx = index * rows + row;  // This should be correct for column-major
```

However, the issue may be in:
1. How matrices are constructed/stored
2. How column vectors are extracted and returned
3. The relationship between matrix storage and column access

## Investigation Steps

1. Trace through `translate_matrix_indexing` to see what values are actually being extracted
2. Verify matrix constructor stores elements correctly in column-major order
3. Check if `load_matrix_column` in `codegen/context.rs` is being used correctly
4. Verify the column extraction loop matches GLSL spec

## Fix Strategy

1. Verify matrix storage order matches GLSL column-major spec
2. Fix column extraction to use correct indices: `col * rows + row` for each row in column
3. Ensure returned vector components match column elements in order
4. Test with mat2, mat3, and mat4 to ensure fix works for all sizes

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/component.rs` - `translate_matrix_indexing`
- `lightplayer/crates/lp-glsl/src/codegen/context.rs` - `load_matrix_column` (if used)
- `lightplayer/crates/lp-glsl/src/codegen/lvalue.rs` - matrix column LValue resolution

## Test Cases

- `matrix/indexing/column-access.glsl` - all tests should pass
- Verify `m[col][row]` still works correctly after fix


