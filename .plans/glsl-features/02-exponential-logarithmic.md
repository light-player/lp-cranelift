# Exponential and Logarithmic Functions Implementation

## Overview

Implement exponential and logarithmic functions from the GLSL specification. These functions are essential for power operations, exponential growth/decay, and logarithmic scaling.

**Spec Reference:** `builtinfunctions.adoc` lines 311-409  
**Priority:** High  
**Estimated Effort:** 1-2 hours

## Current State

- ✅ `sqrt(x)` - fully implemented
- ❌ `pow(x, y)` - registered but returns error (needs exp/log)
- ❌ `exp(x)` - not implemented
- ❌ `log(x)` - not implemented
- ❌ `exp2(x)` - not implemented
- ❌ `log2(x)` - not implemented
- ❌ `inversesqrt(x)` - not implemented

## Requirements

### Functions to Implement

**Exponential Functions:**

- `pow(genFType x, genFType y)` - x raised to y power (x^y)
- `exp(genFType x)` - Natural exponentiation (e^x)
- `exp2(genFType x)` - Base 2 exponentiation (2^x)

**Logarithmic Functions:**

- `log(genFType x)` - Natural logarithm (ln x)
- `log2(genFType x)` - Base 2 logarithm (log₂ x)

**Root Functions:**

- `sqrt(genFType x)` - Square root (✅ already implemented)
- `inversesqrt(genFType x)` - Inverse square root (1/√x)

**Total:** 6 functions (1 already done, 5 to implement)

### Behavior

- All functions operate component-wise on vectors
- `pow(x, y)` = x^y for each component
- `exp(x)` = e^x for each component
- `exp2(x)` = 2^x for each component
- `log(x)` = ln(x) for each component (natural log)
- `log2(x)` = log₂(x) for each component
- `inversesqrt(x)` = 1/√x = faster than `1.0 / sqrt(x)`

### Domain Restrictions

- `log(x)` and `log2(x)` require x > 0
- `pow(x, y)` behavior depends on x and y values
- `sqrt(x)` and `inversesqrt(x)` require x ≥ 0

## Implementation Strategy

### 1. Builtin Registration (`semantic/builtins.rs`)

Update existing `pow` registration and add new functions:

```rust
"pow" => Some(vec![BuiltinSignature {
    name: "pow",
    param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"exp" => Some(vec![BuiltinSignature {
    name: "exp",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"log" => Some(vec![BuiltinSignature {
    name: "log",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"exp2" => Some(vec![BuiltinSignature {
    name: "exp2",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"log2" => Some(vec![BuiltinSignature {
    name: "log2",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"inversesqrt" => Some(vec![BuiltinSignature {
    name: "inversesqrt",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),
```

### 2. Code Generation (`codegen/builtins.rs`)

**For Native Targets (with libm):**

Use Cranelift libcalls:

```rust
fn builtin_pow(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let (y_vals, _) = &args[1];

    if x_vals.len() != y_vals.len() {
        return Err(GlslError::new(ErrorCode::E0104, "pow() requires matching sizes"));
    }

    let mut result_vals = Vec::new();
    for i in 0..x_vals.len() {
        // Use powf(x, y) = expf(y * logf(x))
        // Or use direct powf libcall if available
        let log_x = self.builtin_log(vec![(vec![x_vals[i]], x_ty.clone())])?.0[0];
        let y_times_log_x = self.builder.ins().fmul(y_vals[i], log_x);
        let result = self.builtin_exp(vec![(vec![y_times_log_x], Type::Float)])?.0[0];
        result_vals.push(result);
    }

    Ok((result_vals, x_ty.clone()))
}

fn builtin_exp(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();

    for &val in x_vals {
        // Declare expf if needed
        let sig = self.module.signatures.get(self.exp_sig)?;
        let func_ref = self.module.declare_function("expf", Linkage::Import, sig)?;
        let call_inst = self.builder.ins().call(func_ref, &[val]);
        result_vals.push(call_inst);
    }

    Ok((result_vals, x_ty.clone()))
}

fn builtin_log(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();

    for &val in x_vals {
        let sig = self.module.signatures.get(self.log_sig)?;
        let func_ref = self.module.declare_function("logf", Linkage::Import, sig)?;
        let call_inst = self.builder.ins().call(func_ref, &[val]);
        result_vals.push(call_inst);
    }

    Ok((result_vals, x_ty.clone()))
}

fn builtin_exp2(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();

    for &val in x_vals {
        let sig = self.module.signatures.get(self.exp2_sig)?;
        let func_ref = self.module.declare_function("exp2f", Linkage::Import, sig)?;
        let call_inst = self.builder.ins().call(func_ref, &[val]);
        result_vals.push(call_inst);
    }

    Ok((result_vals, x_ty.clone()))
}

fn builtin_log2(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();

    for &val in x_vals {
        let sig = self.module.signatures.get(self.log2_sig)?;
        let func_ref = self.module.declare_function("log2f", Linkage::Import, sig)?;
        let call_inst = self.builder.ins().call(func_ref, &[val]);
        result_vals.push(call_inst);
    }

    Ok((result_vals, x_ty.clone()))
}

fn builtin_inversesqrt(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();

    for &val in x_vals {
        // Option 1: Use sqrt then divide
        let sqrt_val = self.builder.ins().sqrt(val);
        let one = self.builder.ins().f32const(1.0);
        result_vals.push(self.builder.ins().fdiv(one, sqrt_val));

        // Option 2: Use rsqrtf if available (faster, less accurate)
        // let sig = self.module.signatures.get(self.rsqrt_sig)?;
        // let func_ref = self.module.declare_function("rsqrtf", Linkage::Import, sig)?;
        // result_vals.push(self.builder.ins().call(func_ref, &[val]));
    }

    Ok((result_vals, x_ty.clone()))
}
```

**Function mappings:**

- `pow` → `powf` or `expf(y * logf(x))`
- `exp` → `expf`
- `log` → `logf`
- `exp2` → `exp2f`
- `log2` → `log2f`
- `inversesqrt` → `1.0 / sqrtf(x)` or `rsqrtf(x)` if available

### 3. Code Structure

**File organization:**

- Update `codegen/builtins.rs` (existing file)
- Group exponential functions together
- Group logarithmic functions together
- Reuse existing patterns from `sqrt` implementation

**Error handling:**

- Validate argument count
- Validate argument types (must be GenFType)
- For `pow`, ensure both args have same type/size
- Domain validation can be done at runtime (or compile-time for constants)

## Testing Strategy

### Functionality Tests

**Location:** `crates/lp-glsl-filetests/filetests/exponential/`

**Power Function:**

```glsl
// Test: pow_scalar.glsl
// Spec: builtinfunctions.adoc:275-285
float main() {
    return pow(2.0, 3.0);
}
// run: == 8.0

// Test: pow_vec2.glsl
vec2 main() {
    vec2 x = vec2(2.0, 3.0);
    vec2 y = vec2(2.0, 2.0);
    return pow(x, y);
}
// run: == vec2(4.0, 9.0)
```

**Exponential Functions:**

```glsl
// Test: exp_scalar.glsl
// Spec: builtinfunctions.adoc:286-293
float main() {
    return exp(0.0);
}
// run: == 1.0

// Test: exp_vec3.glsl
vec3 main() {
    vec3 x = vec3(0.0, 1.0, 2.0);
    return exp(x);
}
// run: ≈ vec3(1.0, 2.718282, 7.389056) (within tolerance)

// Test: exp2_scalar.glsl
// Spec: builtinfunctions.adoc:305-312
float main() {
    return exp2(3.0);
}
// run: == 8.0
```

**Logarithmic Functions:**

```glsl
// Test: log_scalar.glsl
// Spec: builtinfunctions.adoc:294-303
float main() {
    float e = 2.718282;
    return log(e);
}
// run: ≈ 1.0 (within tolerance)

// Test: log2_scalar.glsl
// Spec: builtinfunctions.adoc:313-322
float main() {
    return log2(8.0);
}
// run: == 3.0
```

**Inverse Square Root:**

```glsl
// Test: inversesqrt_scalar.glsl
// Spec: builtinfunctions.adoc:337-345
float main() {
    return inversesqrt(4.0);
}
// run: == 0.5

// Test: inversesqrt_vec2.glsl
vec2 main() {
    vec2 x = vec2(4.0, 16.0);
    return inversesqrt(x);
}
// run: == vec2(0.5, 0.25)
```

### Error Handling Tests

**Location:** `crates/lp-glsl-filetests/filetests/type_errors/`

```glsl
// Test: pow_wrong_arg_type.glsl
float main() {
    return pow(true, 2.0);  // ERROR: bool not allowed
}
// EXPECT_ERROR: No matching overload for pow([Bool, Float])

// Test: pow_wrong_arg_count.glsl
float main() {
    return pow(2.0);  // ERROR: missing second arg
}
// EXPECT_ERROR: No matching overload for pow([Float])

// Test: pow_type_mismatch.glsl
float main() {
    return pow(vec2(2.0), 3.0);  // ERROR: type mismatch
}
// EXPECT_ERROR: GenType parameter type mismatch
```

### Edge Cases

```glsl
// Test: pow_zero_exponent.glsl
float main() {
    return pow(5.0, 0.0);  // Any number to 0 = 1
}
// run: == 1.0

// Test: pow_one_exponent.glsl
float main() {
    return pow(5.0, 1.0);  // Any number to 1 = itself
}
// run: == 5.0

// Test: log_one.glsl
float main() {
    return log(1.0);  // ln(1) = 0
}
// run: == 0.0

// Test: log2_one.glsl
float main() {
    return log2(1.0);  // log2(1) = 0
}
// run: == 0.0
```

## Success Criteria

- [ ] All 6 exponential/log functions implemented
- [ ] `pow` works correctly (no longer returns error)
- [ ] Functions registered in `semantic/builtins.rs`
- [ ] Code generation in `codegen/builtins.rs`
- [ ] Scalar versions work with libcalls
- [ ] Vector versions expand component-wise
- [ ] Minimum 12 functionality tests pass
- [ ] Minimum 3 error handling tests pass
- [ ] Edge cases handled correctly
- [ ] Code follows existing patterns and structure
- [ ] No regressions in existing tests

## Future Enhancements

- Direct `powf` libcall if available (faster than exp/log)
- `rsqrtf` for `inversesqrt` if available (faster, less accurate)
- Fixed-point implementations for RISC-V targets
- Constant folding for compile-time values

## Notes

- `pow` can be implemented as `exp(y * log(x))` if `powf` is not available
- `inversesqrt` is commonly optimized in hardware (rsqrt instruction)
- Consider caching function signatures to avoid repeated lookups
- Domain errors (log of negative) are undefined behavior per spec
