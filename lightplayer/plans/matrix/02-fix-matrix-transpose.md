# Fix Matrix Transpose

## Problem

Matrix transpose is producing incorrect results. According to GLSL spec, `transpose(m)[col][row] == m[row][col]`.

**Current behavior:**
- `test_mat3_transpose_verify`: expected `t[1][2] + m[2][1] = 12.0`, got `16.0`
- `test_mat4_transpose`: expected `t[2][3] + m[3][2] = 28.0`, got `30.0`

**Affected tests:**
- `matrix/builtins/transpose.glsl:53` - expected 12.0, got 16.0
- `matrix/builtins/transpose.glsl:69` - expected 28.0, got 30.0

## Root Cause

The transpose implementation in `builtins/matrix.rs` has incorrect index calculation. The current code:
```rust
for result_col in 0..rows {
    for result_row in 0..cols {
        let old_idx = result_row * cols + result_col;
        result_vals.push(m_vals[old_idx]);
    }
}
```

This appears to be using row-major indexing (`result_row * cols + result_col`) instead of column-major.

## Fix Strategy

For column-major storage:
- Input matrix: `m[col][row]` is stored at `col * rows + row`
- Transpose: `result[col][row] = m[row][col]`
- So: `result_vals[result_col * result_rows + result_row] = m_vals[old_col * old_rows + old_row]`
- Where: `old_col = result_row`, `old_row = result_col`

The correct mapping:
```rust
for result_col in 0..rows {  // Transposed matrix has rows columns
    for result_row in 0..cols {  // Transposed matrix has cols rows
        // result[result_col][result_row] = m[result_row][result_col]
        // m[result_row][result_col] = m_vals[result_row * cols + result_col]
        let old_idx = result_row * cols + result_col;
        result_vals.push(m_vals[old_idx]);
    }
}
```

Wait, that's what the code already does. The issue might be:
1. The loop bounds are wrong (should be `0..cols` then `0..rows`?)
2. The index calculation is wrong
3. The matrix dimensions are swapped incorrectly

Actually, for a mat3 (3x3):
- Input: 3 rows, 3 cols
- Transpose: 3 rows, 3 cols (same size)
- But conceptually: transpose swaps rows and columns

Let me reconsider: if `m` is stored column-major as `[col0_row0, col0_row1, ..., col1_row0, ...]`, then:
- `m[col][row]` is at index `col * rows + row`
- After transpose, `t[col][row] = m[row][col]`
- `m[row][col]` is at index `row * cols + col` (wait, that's row-major)
- Actually: `m[row][col]` in column-major is at `col * rows + row`... no wait

The correct formula for column-major:
- `m[col][row]` is stored at `col * rows + row`
- To get `m[row][col]`, we need to find which column and row that is
- `m[row][col]` means column `row`, row `col` in the original matrix
- So it's stored at `row * rows + col`... no, that's wrong

Actually: In GLSL, `m[col][row]` means:
- First index `col` selects the column
- Second index `row` selects the row within that column
- Storage: column-major means columns are stored contiguously
- So `m[col][row]` is at `col * rows + row` ✓

For transpose: `t[col][row] = m[row][col]`
- `m[row][col]` means column `row`, row `col` in original
- Stored at `row * rows + col`... wait no

Let me think step by step:
- Original matrix `m`: `m[col][row]` at index `col * rows + row`
- Transpose: `t[col][row] = m[row][col]`
- `m[row][col]` means: column `row`, row `col` in original
- Original storage: column `row` starts at `row * rows`, then add `col` for the row within that column
- So: `m[row][col]` is at `row * rows + col`... but that's wrong because `col` is the row index

I think the confusion is: `m[row][col]` in GLSL notation means column `row`, row `col`. But the storage is column-major, so:
- Column `row` has `rows` elements
- Column `row` starts at index `row * rows`
- Within column `row`, row `col` is at offset `col`
- So `m[row][col]` is at `row * rows + col`

But wait, that would mean `m[1][0]` (column 1, row 0) is at `1 * rows + 0 = rows`, which matches!

So for transpose:
- `t[col][row] = m[row][col]`
- `m[row][col]` is at `row * rows + col` (in original storage)
- `t[col][row]` should be stored at `col * rows + row` (in result storage)

So the correct code:
```rust
for result_col in 0..rows {  // Result has rows columns (transposed)
    for result_row in 0..cols {  // Result has cols rows (transposed)
        // t[result_col][result_row] = m[result_row][result_col]
        // m[result_row][result_col] is at: result_row * rows + result_col
        let old_idx = result_row * rows + result_col;
        result_vals.push(m_vals[old_idx]);
    }
}
```

But wait, the current code uses `result_row * cols + result_col`. That's the bug!

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/builtins/matrix.rs` - `builtin_transpose`

## Test Cases

- `matrix/builtins/transpose.glsl` - all tests should pass
- Verify transpose works for mat2, mat3, mat4

