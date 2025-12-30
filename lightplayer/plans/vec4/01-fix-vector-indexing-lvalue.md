# Fix Vector Indexing as LValue

## Problem

Vector array indexing `v[0] = float` is not supported as an LValue. The error message says "vector indexing not supported as LValue (use component access like .x)".

**Current behavior:**

- `v[0] = 100.0` fails with compilation error
- `v.x = 100.0` works correctly

**Affected tests:**

- `vec4/assignment/element-assignment.glsl:58` - `test_vec4_element_assignment_array_index()` fails

## Root Cause

In `lightplayer/crates/lp-glsl/src/codegen/lvalue.rs`, the `resolve_lvalue` function returns an error when encountering vector indexing:

```rust
// This is vector component access - not supported as LValue yet
// (vectors are typically accessed via .x, .y, etc.)
return Err(GlslError::new(
    ErrorCode::E0400,
    "vector indexing not supported as LValue (use component access like .x)",
)
.with_location(source_span_to_location(span)));
```

However, GLSL spec allows `v[0]` to be used as an LValue, equivalent to `v.x`.

## Fix Strategy

1. **Add VectorElement LValue variant** similar to `MatrixElement`:

   - Store base variables, base type, and component index
   - Support single-element access like `v[0]`

2. **Update resolve_lvalue** to create `VectorElement` LValue:

   - When indexing a vector, create `VectorElement` instead of erroring
   - Map index to component: `[0]` → x, `[1]` → y, `[2]` → z, `[3]` → w

3. **Implement write_lvalue** for `VectorElement`:

   - Write single value to the appropriate component variable
   - Similar to how `Component` LValue works but for single index

4. **Support read_lvalue** for `VectorElement`:
   - Read single component value from vector

## Implementation Steps

1. **Add VectorElement variant** to `LValue` enum in `lvalue.rs`:

   ```rust
   VectorElement {
       base_vars: Vec<Variable>,
       base_ty: GlslType,
       index: usize,  // Component index (0=x, 1=y, 2=z, 3=w)
   }
   ```

2. **Update resolve_lvalue** in `lvalue.rs`:

   - When `current_ty.is_vector()` and we have a single index, create `VectorElement`
   - Remove the error case for vector indexing

3. **Implement read_lvalue** for `VectorElement`:

   - Return the value from `base_vars[index]`

4. **Implement write_lvalue** for `VectorElement`:

   - Write single value to `base_vars[index]`

5. **Update any code** that pattern matches on `LValue` to handle `VectorElement`

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/lvalue.rs` - Add `VectorElement` variant and implement handlers
- Check for any other files that pattern match on `LValue` enum

## Test Cases

- `vec4/assignment/element-assignment.glsl` - `test_vec4_element_assignment_array_index()` should pass
- Verify `v[0] = x` works for all indices (0, 1, 2, 3)
- Verify `v[0]` read access still works (should already work)

## Acceptance Criteria

- [ ] `test_vec4_element_assignment_array_index()` passes
- [ ] All tests in `vec4/assignment/element-assignment.glsl` pass
- [ ] No regressions in other vec4 tests
- [ ] Code compiles without warnings

## Verification

Run the test:

```bash
scripts/glsl-filetests.sh vec4/assignment/element-assignment.glsl:58
```

Expected result: Test passes with no errors.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement vector indexing as LValue"
```




