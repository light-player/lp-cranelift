# Fix Outer Product

## Problem

The `outerProduct(c, r)` function is producing incorrect results. According to GLSL spec, `outerProduct(c, r)[col][row] == c[col] * r[row]`.

**Current behavior:**
- `test_outer_product_vec4`: expected sum of column 0 = `26.0` (5+6+7+8), got `50.0` (5+10+15+20)
- The pattern `50.0 = 5+10+15+20` suggests it's computing `c[i] * r[0]` instead of `c[0] * r[i]`

**Affected tests:**
- `matrix/builtins/outer-product.glsl:55` - expected 26.0, got 50.0

## Root Cause

The implementation in `builtins/matrix.rs` has:
```rust
for j in 0..vec2_size {
    for i in 0..vec1_size {
        let product = self.builder.ins().fmul(vec1_vals[i], vec2_vals[j]);
        result_vals.push(product);
    }
}
```

The comment says `result[i][j] = vec1[i] * vec2[j]`, but according to GLSL spec:
- `outerProduct(c, r)[col][row] = c[col] * r[row]`
- So `result[col][row] = c[col] * r[row]`

The current code computes `vec1[i] * vec2[j]` where:
- `i` ranges over `vec1_size` (columns)
- `j` ranges over `vec2_size` (rows)

But the loop order is `j` (outer) then `i` (inner), which means:
- First iteration: `j=0, i=0..vec1_size` → `c[0]*r[0], c[1]*r[0], c[2]*r[0], ...`
- This produces columns where each column has `c[i] * r[0]`, which is wrong!

The correct order should be:
- For column `col`: `result[col][row] = c[col] * r[row]` for all `row`
- So for each `col`, we need all `row` values
- Storage is column-major, so we iterate columns first, then rows within each column

The fix:
```rust
for col in 0..vec1_size {  // Columns come from vec1
    for row in 0..vec2_size {  // Rows come from vec2
        // result[col][row] = c[col] * r[row]
        let product = self.builder.ins().fmul(vec1_vals[col], vec2_vals[row]);
        result_vals.push(product);
    }
}
```

## Fix Strategy

1. Change loop order: iterate columns (from vec1) first, then rows (from vec2) within each column
2. Update formula: `result[col][row] = c[col] * r[row]`
3. Verify the result matrix dimensions match GLSL spec

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/builtins/matrix.rs` - `builtin_outerProduct`

## Test Cases

- `matrix/builtins/outer-product.glsl` - all tests should pass
- Verify outerProduct works for vec2×vec2, vec3×vec3, vec4×vec4


