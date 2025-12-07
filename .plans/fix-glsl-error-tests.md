# Plan: Fix Failing GLSL Filetests

## Overview

8 GLSL filetests are failing due to missing location information in error messages and one assertion failure.

## Issues Identified

### 1. Missing Location Information (7 tests)

Most error tests expect `EXPECT_LOCATION` to match a line number, but errors are returning `<unknown>` instead.

**Affected Tests:**

- `test_function_return_type_mismatch_error`
- `test_function_wrong_arg_type_error`
- `test_return_type_mismatch_error`
- `test_swizzle_invalid_component_error`
- `test_unsupported_type_error`
- `test_vec_add_wrong_size_error`
- `test_vec_component_out_of_range_error`

**Root Causes:**

1. **Component access errors** (`parse_vector_swizzle`): Errors created without span information
2. **Vector operation type mismatch**: Errors created without span from binary operation
3. **Unsupported type errors**: Errors created during type parsing without span
4. **Return type mismatch**: Span exists but may not be properly attached
5. **Function argument type mismatch**: Location may be incorrect

### 2. Assertion Failure (1 test)

- `test_vec3_function`: Hits `assertion failed: func_ctx.is_empty()` in `FunctionBuilder::new()`

**Root Cause:**
`FunctionBuilderContext` is reused across function compilations but not properly cleared. The context maintains state (variables, blocks) that persists between function compilations.

## Fix Plan

### Fix 1: Add Location to Component Access Errors

**File:** `crates/lp-glsl/src/codegen/expr.rs`

- Modify `parse_vector_swizzle` to accept an optional `span` parameter
- Pass span from `Expr::Dot` handler (line 170) to `parse_vector_swizzle`
- Add location to component out-of-range errors (line 1246)

```rust
fn parse_vector_swizzle(name: &str, vec_ty: &GlslType, span: Option<glsl::syntax::SourceSpan>) -> Result<Vec<usize>, GlslError> {
    // ... existing code ...
    if idx >= component_count {
        let mut error = GlslError::new(ErrorCode::E0111, format!(
            "component '{}' not valid for {:?} (has only {} components)",
            ch, vec_ty, component_count
        ));
        if let Some(s) = span {
            error = error.with_location(source_span_to_location(&s));
        }
        return Err(error);
    }
    // ...
}
```

### Fix 2: Add Location to Vector Operation Type Mismatch

**File:** `crates/lp-glsl/src/codegen/expr.rs`

- In `translate_vector_binary_op` (line 414), capture span from binary operation
- Add location to type mismatch error (line 417)

```rust
fn translate_vector_binary_op(
    &mut self,
    op: &glsl::syntax::BinaryOp,
    lhs_vals: Vec<Value>,
    lhs_ty: &GlslType,
    rhs_vals: Vec<Value>,
    rhs_ty: &GlslType,
    span: Option<glsl::syntax::SourceSpan>, // Add span parameter
) -> Result<(Vec<Value>, GlslType), GlslError> {
    // ...
    if lhs_ty != rhs_ty {
        let mut error = GlslError::new(ErrorCode::E0106, format!(
            "vector operation requires matching types, got {:?} and {:?}",
            lhs_ty, rhs_ty
        ));
        if let Some(s) = span {
            error = error.with_location(source_span_to_location(&s));
        }
        return Err(error);
    }
    // ...
}
```

### Fix 3: Add Location to Unsupported Type Errors

**File:** `crates/lp-glsl/src/semantic/mod.rs`

- Modify `parse_type_specifier` to accept optional span
- Pass span from `parse_return_type` and function parameter parsing
- Add location to unsupported type error (line 171)

```rust
fn parse_type_specifier(ty: &glsl::syntax::TypeSpecifierNonArray, span: Option<glsl::syntax::SourceSpan>) -> Result<types::Type, GlslError> {
    match ty.ty {
        // ... existing cases ...
        _ => {
            let mut error = GlslError::unsupported_type(format!("{:?}", ty.ty));
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            Err(error)
        }
    }
}
```

### Fix 4: Verify Return Type Mismatch Location

**File:** `crates/lp-glsl/src/codegen/stmt.rs`

- Verify that `translate_return` properly passes span to `coerce_to_type_with_location`
- Ensure span is extracted from return expression (line 322)

**Status:** Already has span extraction, but verify it's being used correctly.

### Fix 5: Fix Function Argument Type Mismatch Location

**File:** `crates/lp-glsl/src/codegen/expr.rs`

- Verify location is correctly set in `translate_user_function_call` (line 1165)
- Ensure error uses the call span, not parameter span

**Status:** Already has location at line 1165, but may need to verify it matches test expectations.

### Fix 6: Clear FunctionBuilderContext Between Compilations

**File:** `crates/lp-glsl/src/jit.rs`

- After `builder.finalize()` in `compile_function` (line 471), ensure builder is dropped
- Before creating new `FunctionBuilder` in `compile_main_function` (line 522), verify context is empty
- Consider creating a new `FunctionBuilderContext` for each function, or add explicit clearing

**Options:**

1. Create new `FunctionBuilderContext` for each function (simpler, but allocates)
2. Add method to clear context state (if available in cranelift)
3. Ensure builder is properly dropped before reuse

**Recommended:** Create new context for each function compilation to avoid state pollution.

## Implementation Order

1. Fix 6 (assertion failure) - highest priority, blocks compilation
2. Fix 1 (component access) - affects 2 tests
3. Fix 2 (vector operations) - affects 1 test
4. Fix 3 (unsupported type) - affects 1 test
5. Fix 4 & 5 (verify existing fixes) - affects 3 tests

## Testing

After each fix, run:

```bash
cargo test --package lp-glsl-filetests --test filetests
```

Use BLESS mode to update expectations if error messages change:

```bash
CRANELIFT_TEST_BLESS=1 cargo test --package lp-glsl-filetests --test filetests
```

## Notes

- Location information should use line numbers (1-indexed) as expected by tests
- Some errors may be caught at different phases (semantic vs codegen), ensure location is preserved
- The `func_ctx.is_empty()` assertion is a debug assertion, so it may not fail in release builds
