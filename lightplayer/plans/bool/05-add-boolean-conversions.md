# Phase 5: Add Boolean Type Conversions

## Goal

Support all boolean type conversions in the coercion module: `bool ↔ int`, `bool ↔ float`, `bool ↔ uint`.

## Problem

The coercion module only handles `int → float` conversion:

```rust
// In codegen/expr/coercion.rs
match (from_ty, to_ty) {
    (GlslType::Int, GlslType::Float) => { ... }
    _ => Err(...)  // All other conversions fail
}
```

**Missing Conversions**:

- `bool → int`: false → 0, true → 1
- `bool → float`: false → 0.0, true → 1.0
- `bool → uint`: false → 0u, true → 1u
- `int → bool`: 0 → false, non-zero → true
- `float → bool`: 0.0 → false, non-zero → true
- `uint → bool`: 0 → false, non-zero → true

**GLSL Spec**: All conversions via constructors are explicit conversions:

- `bool(value)`: 0/0.0 → false, non-zero → true
- `int(bool)`: false → 0, true → 1
- `float(bool)`: false → 0.0, true → 1.0
- `uint(bool)`: false → 0u, true → 1u

## Solution

Add all boolean conversion cases to the coercion module.

## Implementation Steps

### Step 1: Add Boolean to Numeric Conversions

**File**: `lightplayer/crates/lp-glsl/src/codegen/expr/coercion.rs`

Add conversions from bool to numeric types:

```rust
match (from_ty, to_ty) {
    (GlslType::Int, GlslType::Float) => {
        // int → float: fcvt_from_sint
        Ok(ctx.builder.ins().fcvt_from_sint(types::F32, val))
    }
    // Boolean to numeric conversions
    (GlslType::Bool, GlslType::Int) => {
        // bool → int: false → 0, true → 1
        // val is i8 (0 or 1), extend to i32
        Ok(ctx.builder.ins().uextend(types::I32, val))
    }
    (GlslType::Bool, GlslType::Float) => {
        // bool → float: false → 0.0, true → 1.0
        // val is i8 (0 or 1), convert to i32 then to float
        let i32_val = ctx.builder.ins().uextend(types::I32, val);
        Ok(ctx.builder.ins().fcvt_from_sint(types::F32, i32_val))
    }
    (GlslType::Bool, GlslType::UInt) => {
        // bool → uint: false → 0u, true → 1u
        // val is i8 (0 or 1), extend to i32 (uint is represented as i32)
        Ok(ctx.builder.ins().uextend(types::I32, val))
    }
    // ... rest of conversions ...
}
```

### Step 2: Add Numeric to Boolean Conversions

Add conversions from numeric types to bool:

```rust
    // Numeric to boolean conversions
    (GlslType::Int, GlslType::Bool) => {
        // int → bool: 0 → false, non-zero → true
        // val is i32, compare with 0, convert result to i8
        let zero = ctx.builder.ins().iconst(types::I32, 0);
        let cmp = ctx.builder.ins().icmp(IntCC::NotEqual, val, zero);
        let one = ctx.builder.ins().iconst(types::I8, 1);
        let zero_i8 = ctx.builder.ins().iconst(types::I8, 0);
        Ok(ctx.builder.ins().select(cmp, one, zero_i8))
    }
    (GlslType::Float, GlslType::Bool) => {
        // float → bool: 0.0 → false, non-zero → true
        // val is f32, compare with 0.0, convert result to i8
        let zero = ctx.builder.ins().f32const(0.0);
        let cmp = ctx.builder.ins().fcmp(FloatCC::NotEqual, val, zero);
        let one = ctx.builder.ins().iconst(types::I8, 1);
        let zero_i8 = ctx.builder.ins().iconst(types::I8, 0);
        Ok(ctx.builder.ins().select(cmp, one, zero_i8))
    }
    (GlslType::UInt, GlslType::Bool) => {
        // uint → bool: 0 → false, non-zero → true
        // val is i32 (uint represented as i32), compare with 0, convert result to i8
        let zero = ctx.builder.ins().iconst(types::I32, 0);
        let cmp = ctx.builder.ins().icmp(IntCC::NotEqual, val, zero);
        let one = ctx.builder.ins().iconst(types::I8, 1);
        let zero_i8 = ctx.builder.ins().iconst(types::I8, 0);
        Ok(ctx.builder.ins().select(cmp, one, zero_i8))
    }
```

### Step 3: Add Missing Imports

Ensure all necessary imports are present:

```rust
use cranelift_codegen::ir::{types, Value, InstBuilder, IntCC, FloatCC};
```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/codegen/expr/coercion.rs` - Add all boolean conversions

## Test Cases

All conversion tests should pass:

- `bool/from-bool.glsl` - `bool(bool)` (no-op, but tests constructor)
- `bool/from-int.glsl` - `bool(int)` constructor
- `bool/from-float.glsl` - `bool(float)` constructor
- `bool/from-uint.glsl` - `bool(uint)` constructor
- `bool/to-int.glsl` - `int(bool)` constructor
- `bool/to-float.glsl` - `float(bool)` constructor
- `bool/to-uint.glsl` - `uint(bool)` constructor
- `bool/assign-simple.glsl` - `bool(5)` constructor

## Expected Behavior

**Numeric to Boolean:**

- `bool(0)` → `false`
- `bool(5)` → `true`
- `bool(-10)` → `true`
- `bool(0.0)` → `false`
- `bool(3.14)` → `true`

**Boolean to Numeric:**

- `int(false)` → `0`
- `int(true)` → `1`
- `float(false)` → `0.0`
- `float(true)` → `1.0`
- `uint(false)` → `0u`
- `uint(true)` → `1u`

## Verification

Run all conversion tests:

```bash
scripts/glsl-filetests.sh bool/from-int.glsl
scripts/glsl-filetests.sh bool/from-float.glsl
scripts/glsl-filetests.sh bool/from-uint.glsl
scripts/glsl-filetests.sh bool/to-int.glsl
scripts/glsl-filetests.sh bool/to-float.glsl
scripts/glsl-filetests.sh bool/to-uint.glsl
scripts/glsl-filetests.sh bool/assign-simple.glsl
```

Expected result: All conversion tests pass.

## Commit Instructions

Once tests pass:

```bash
git add -A
git commit -m "lpc: add boolean type conversions"
```

## Notes

- **Boolean Representation**: Booleans are stored as i8 (0 = false, 1 = true)
- **UInt Representation**: UInt is represented as i32 in Cranelift
- **Zero Comparison**: For numeric → bool, any non-zero value converts to true
- **Explicit Conversions**: All conversions are explicit (via constructors), not implicit




