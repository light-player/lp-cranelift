# Phase 4: Add Scalar Type Constructors

## Goal

Support scalar type constructors like `bool(int)`, `int(bool)`, `float(bool)`, `uint(bool)`.

## Problem

Function call handler only checks for vector/matrix constructors, not scalar constructors:

```rust
// In codegen/expr/function.rs
if is_vector_type_name(func_name) {
    return constructor::translate_vector_constructor(...);
}
if is_matrix_type_name(func_name) {
    return constructor::translate_matrix_constructor(...);
}
// No handling for bool(int), int(bool), etc.
```

**Error**: `error[E0101]: undefined function 'bool'`

**GLSL Spec**: Scalar type constructors allow explicit type conversion:
- `bool(value)`: Converts numeric types to bool (0/0.0 → false, non-zero → true)
- `int(bool)`: Converts bool to int (false → 0, true → 1)
- `float(bool)`: Converts bool to float (false → 0.0, true → 1.0)
- `uint(bool)`: Converts bool to uint (false → 0u, true → 1u)

## Solution

Add scalar constructor handling in the function call dispatcher, similar to vector/matrix constructors.

## Implementation Steps

### Step 1: Add Scalar Constructor Detection

**File**: `lightplayer/crates/lp-glsl/src/semantic/type_check/mod.rs` or create new file

Add helper function to detect scalar type names:

```rust
pub fn is_scalar_type_name(name: &str) -> bool {
    matches!(name, "bool" | "int" | "float" | "uint")
}
```

### Step 2: Add Scalar Constructor Translation

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs`

Add function to translate scalar constructors:

```rust
/// Translate scalar type constructor (bool(int), int(bool), etc.)
pub fn translate_scalar_constructor(
    ctx: &mut CodegenContext,
    type_name: &str,
    args: &[Expr],
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    // Ensure we're in a block before evaluating
    ctx.ensure_block()?;
    
    // Scalar constructors take exactly one argument
    if args.len() != 1 {
        return Err(GlslError::new(
            ErrorCode::E0115,
            format!("`{}` constructor requires exactly one argument", type_name),
        )
        .with_location(source_span_to_location(&span)));
    }
    
    // Translate argument
    let (arg_vals, arg_ty) = ctx.translate_expr_typed(&args[0])?;
    if arg_vals.len() != 1 {
        return Err(GlslError::new(
            ErrorCode::E0115,
            format!("`{}` constructor requires scalar argument", type_name),
        )
        .with_location(source_span_to_location(&span)));
    }
    let arg_val = arg_vals[0];
    
    // Determine result type
    let result_ty = match type_name {
        "bool" => GlslType::Bool,
        "int" => GlslType::Int,
        "float" => GlslType::Float,
        "uint" => GlslType::UInt,
        _ => {
            return Err(GlslError::new(
                ErrorCode::E0112,
                format!("`{}` is not a scalar type", type_name),
            )
            .with_location(source_span_to_location(&span)));
        }
    };
    
    // Convert argument to result type using coercion
    let result_val = coercion::coerce_to_type_with_location(
        ctx,
        arg_val,
        &arg_ty,
        &result_ty,
        Some(span.clone()),
    )?;
    
    Ok((vec![result_val], result_ty))
}
```

### Step 3: Add Scalar Constructor Dispatch

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/function.rs`

Add scalar constructor check in `translate_function_call`:

```rust
use crate::semantic::type_check::{is_matrix_type_name, is_vector_type_name, is_scalar_type_name};

pub fn translate_function_call(...) -> Result<...> {
    // ... existing code ...
    
    // Check if it's a type constructor
    if is_vector_type_name(func_name) {
        return constructor::translate_vector_constructor(ctx, func_name, args, span.clone());
    }

    if is_matrix_type_name(func_name) {
        return constructor::translate_matrix_constructor(ctx, func_name, args);
    }
    
    // NEW: Check for scalar constructors
    if is_scalar_type_name(func_name) {
        return constructor::translate_scalar_constructor(ctx, func_name, args, span.clone());
    }

    // Check if it's a built-in function
    // ... rest of function ...
}
```

### Step 4: Export Helper Function

**File**: `lightplayer/crates/lp-glsl/src/semantic/type_check/mod.rs`

Export the helper function:

```rust
pub fn is_scalar_type_name(name: &str) -> bool {
    matches!(name, "bool" | "int" | "float" | "uint")
}
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/semantic/type_check/mod.rs` - Add `is_scalar_type_name` helper
- `lightplayer/crates/lp-glsl/src/codegen/expr/constructor.rs` - Add `translate_scalar_constructor`
- `lightplayer/crates/lp-glsl/src/codegen/expr/function.rs` - Add scalar constructor dispatch

## Test Cases

All constructor/conversion tests should pass:
- `bool/assign-simple.glsl` - `bool(5)` constructor
- `bool/from-bool.glsl` - `bool(bool)` constructor
- `bool/from-int.glsl` - `bool(int)` constructor
- `bool/from-float.glsl` - `bool(float)` constructor
- `bool/from-uint.glsl` - `bool(uint)` constructor
- `bool/to-int.glsl` - `int(bool)` constructor
- `bool/to-float.glsl` - `float(bool)` constructor
- `bool/to-uint.glsl` - `uint(bool)` constructor

## Expected Behavior

- `bool(0)` → `false`
- `bool(5)` → `true`
- `bool(-10)` → `true`
- `int(false)` → `0`
- `int(true)` → `1`
- `float(false)` → `0.0`
- `float(true)` → `1.0`

## Verification

Run constructor/conversion tests:

```bash
scripts/glsl-filetests.sh bool/assign-simple.glsl
scripts/glsl-filetests.sh bool/from-int.glsl
scripts/glsl-filetests.sh bool/to-int.glsl
```

Expected result: Constructor tests pass (conversion tests may still fail until Phase 5 adds the actual conversions).

## Commit Instructions

Once tests pass:

```bash
git add -A
git commit -m "lpc: add scalar type constructor support"
```

## Notes

- **Dependency on Phase 5**: This phase adds the constructor infrastructure, but actual conversions depend on Phase 5 (type conversions). Some tests may still fail until conversions are implemented.
- **Single Argument**: Scalar constructors always take exactly one argument (unlike vector/matrix constructors which can take multiple)
- **Uses Coercion**: The implementation delegates to the coercion module, which will need to be extended in Phase 5





