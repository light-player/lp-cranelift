# Trigonometric Functions Implementation

## Overview

Implement all angle and trigonometry functions from the GLSL specification. These functions are critical for shader effects involving rotations, oscillations, and periodic patterns.

**Spec Reference:** `builtinfunctions.adoc` lines 122-310  
**Priority:** High  
**Estimated Effort:** 2-3 hours

## Current State

- ❌ No trigonometric functions implemented
- ✅ Fixed-point transformation available (can be used for approximations)
- ✅ Pattern exists for libcalls (see `sqrt` implementation)

## Requirements

### Functions to Implement

**Angle Conversion:**

- `radians(genFType degrees)` - Convert degrees to radians
- `degrees(genFType radians)` - Convert radians to degrees

**Basic Trigonometry:**

- `sin(genFType angle)` - Sine
- `cos(genFType angle)` - Cosine
- `tan(genFType angle)` - Tangent

**Inverse Trigonometry:**

- `asin(genFType x)` - Arc sine (returns angle in radians)
- `acos(genFType x)` - Arc cosine (returns angle in radians)
- `atan(genFType y_over_x)` - Arc tangent (1-arg, returns angle in radians)
- `atan(genFType y, genFType x)` - Arc tangent (2-arg, returns angle in radians)

**Hyperbolic Functions:**

- `sinh(genFType x)` - Hyperbolic sine
- `cosh(genFType x)` - Hyperbolic cosine
- `tanh(genFType x)` - Hyperbolic tangent
- `asinh(genFType x)` - Inverse hyperbolic sine
- `acosh(genFType x)` - Inverse hyperbolic cosine
- `atanh(genFType x)` - Inverse hyperbolic tangent

**Total:** 15 functions

### Behavior

- All functions operate component-wise on vectors
- Input angles are in radians (except `radians()` which takes degrees)
- Output angles are in radians (except `degrees()` which outputs degrees)
- Domain restrictions apply (e.g., `asin`/`acos` require input in [-1, 1])
- No divide-by-zero errors (spec requirement)

## Implementation Strategy

### 1. Builtin Registration (`semantic/builtins.rs`)

Add signatures for all 15 functions:

```rust
"radians" => Some(vec![BuiltinSignature {
    name: "radians",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),
// ... repeat for all functions
```

**Special cases:**

- `atan` has two overloads (1-arg and 2-arg)
- All use `GenFType` (float, vec2, vec3, vec4)

### 2. Code Generation (`codegen/builtins.rs`)

**For Native Targets (with libm):**

Use Cranelift libcalls to standard math library functions:

```rust
fn builtin_sin(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();

    for &val in x_vals {
        // Declare external function if not already declared
        let sig = self.module.signatures.get(self.sin_sig)?;
        let func_ref = self.module.declare_function("sinf", Linkage::Import, sig)?;
        let call_inst = self.builder.ins().call(func_ref, &[val]);
        result_vals.push(call_inst);
    }

    Ok((result_vals, x_ty.clone()))
}
```

**Function mappings:**

- `sin` → `sinf`
- `cos` → `cosf`
- `tan` → `tanf`
- `asin` → `asinf`
- `acos` → `acosf`
- `atan` (1-arg) → `atanf`
- `atan` (2-arg) → `atan2f`
- `sinh` → `sinhf`
- `cosh` → `coshf`
- `tanh` → `tanhf`
- `asinh` → `asinhf`
- `acosh` → `acoshf`
- `atanh` → `atanhf`

**For Fixed-Point/RISC-V:**

For now, return an error indicating fixed-point versions need runtime library. Future work can add CORDIC or lookup table implementations.

### 3. Angle Conversion Functions

These are simple arithmetic operations:

```rust
fn builtin_radians(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (deg_vals, deg_ty) = &args[0];
    let pi_over_180 = self.builder.ins().f32const(0.017453292519943295); // π/180
    let mut result_vals = Vec::new();

    for &deg in deg_vals {
        result_vals.push(self.builder.ins().fmul(deg, pi_over_180));
    }

    Ok((result_vals, deg_ty.clone()))
}

fn builtin_degrees(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (rad_vals, rad_ty) = &args[0];
    let _180_over_pi = self.builder.ins().f32const(57.29577951308232); // 180/π
    let mut result_vals = Vec::new();

    for &rad in rad_vals {
        result_vals.push(self.builder.ins().fmul(rad, _180_over_pi));
    }

    Ok((result_vals, rad_ty.clone()))
}
```

### 4. Code Structure

**File organization:**

- Add to `codegen/builtins.rs` (existing file)
- Group related functions together
- Use helper functions for common patterns (libcall generation)

**Error handling:**

- Validate argument count
- Validate argument types (must be GenFType)
- For `atan` 2-arg version, ensure both args have same type/size

## Testing Strategy

### Functionality Tests

**Location:** `crates/lp-glsl-filetests/filetests/trig/`

**Basic Trigonometry:**

```glsl
// Test: sin_scalar.glsl
// Spec: builtinfunctions.adoc:149-156
float main() {
    return sin(0.0);
}
// run: == 0.0

// Test: cos_scalar.glsl
// Spec: builtinfunctions.adoc:157-164
float main() {
    return cos(0.0);
}
// run: == 1.0

// Test: tan_scalar.glsl
// Spec: builtinfunctions.adoc:165-172
float main() {
    float pi_4 = 0.785398163;  // π/4
    return tan(pi_4);
}
// run: ≈ 1.0 (within 0.001 tolerance)

// Test: sin_vec3.glsl
// Spec: builtinfunctions.adoc:149-156 - component-wise
vec3 main() {
    vec3 angles = vec3(0.0, 1.570796327, 3.141592654); // 0, π/2, π
    return sin(angles);
}
// run: ≈ vec3(0.0, 1.0, 0.0) (within tolerance)
```

**Angle Conversion:**

```glsl
// Test: radians_degrees.glsl
// Spec: builtinfunctions.adoc:133-147
float main() {
    float deg = 180.0;
    float rad = radians(deg);
    return degrees(rad);
}
// run: == 180.0

// Test: radians_vec2.glsl
vec2 main() {
    vec2 deg = vec2(90.0, 45.0);
    return radians(deg);
}
// run: ≈ vec2(1.570796327, 0.785398163) (within tolerance)
```

**Inverse Functions:**

```glsl
// Test: asin_scalar.glsl
// Spec: builtinfunctions.adoc:173-183
float main() {
    return asin(0.5);
}
// run: ≈ 0.523599 (π/6, within tolerance)

// Test: acos_scalar.glsl
// Spec: builtinfunctions.adoc:184-195
float main() {
    return acos(0.5);
}
// run: ≈ 1.047198 (π/3, within tolerance)

// Test: atan_scalar.glsl
// Spec: builtinfunctions.adoc:196-203
float main() {
    return atan(1.0);
}
// run: ≈ 0.785398 (π/4, within tolerance)

// Test: atan2_scalar.glsl
// Spec: builtinfunctions.adoc:204-211
float main() {
    return atan(1.0, 1.0);
}
// run: ≈ 0.785398 (π/4, within tolerance)
```

**Hyperbolic Functions:**

```glsl
// Test: sinh_scalar.glsl
// Spec: builtinfunctions.adoc:229-237
float main() {
    return sinh(0.0);
}
// run: == 0.0

// Test: cosh_scalar.glsl
// Spec: builtinfunctions.adoc:238-246
float main() {
    return cosh(0.0);
}
// run: == 1.0

// Test: tanh_scalar.glsl
// Spec: builtinfunctions.adoc:247-255
float main() {
    return tanh(0.0);
}
// run: == 0.0
```

### Error Handling Tests

**Location:** `crates/lp-glsl-filetests/filetests/type_errors/`

```glsl
// Test: trig_wrong_arg_type.glsl
float main() {
    return sin(true);  // ERROR: bool not allowed
}
// EXPECT_ERROR: No matching overload for sin([Bool])

// Test: trig_wrong_arg_count.glsl
float main() {
    return sin(1.0, 2.0);  // ERROR: too many args
}
// EXPECT_ERROR: No matching overload for sin([Float, Float])

// Test: atan2_type_mismatch.glsl
float main() {
    return atan(vec2(1.0), 2.0);  // ERROR: type mismatch
}
// EXPECT_ERROR: GenType parameter type mismatch
```

### Edge Cases

```glsl
// Test: sin_pi.glsl
float main() {
    float pi = 3.141592654;
    return sin(pi);
}
// run: ≈ 0.0 (within tolerance)

// Test: cos_pi.glsl
float main() {
    float pi = 3.141592654;
    return cos(pi);
}
// run: ≈ -1.0 (within tolerance)

// Test: asin_boundary.glsl
float main() {
    return asin(1.0);  // Should return π/2
}
// run: ≈ 1.570796327 (within tolerance)
```

## Success Criteria

- [ ] All 15 trigonometric functions implemented
- [ ] Functions registered in `semantic/builtins.rs`
- [ ] Code generation in `codegen/builtins.rs`
- [ ] Scalar versions work with libcalls
- [ ] Vector versions expand component-wise
- [ ] Angle conversion functions use arithmetic (no libcalls)
- [ ] Minimum 20 functionality tests pass
- [ ] Minimum 5 error handling tests pass
- [ ] Edge cases handled correctly
- [ ] Code follows existing patterns and structure
- [ ] No regressions in existing tests

## Future Enhancements

- Fixed-point implementations for RISC-V targets
- CORDIC algorithm for hardware without FPU
- Lookup tables for common angles
- SIMD optimizations for vector operations

## Notes

- Angle conversion functions (`radians`, `degrees`) are simple arithmetic and should be implemented first
- For `atan` 2-arg version, use `atan2f` which handles quadrant correctly
- Consider caching function signatures to avoid repeated lookups
- Fixed-point versions can be added later as a separate pass
