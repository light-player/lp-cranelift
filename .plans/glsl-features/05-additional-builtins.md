# Additional Built-in Functions Implementation

## Overview

Implement additional common and geometric functions that are useful but less critical than core math functions. These complete the standard GLSL built-in function set.

**Spec Reference:** `builtinfunctions.adoc` lines 400-771 (Common Functions), 1062-1115 (Geometric Functions)  
**Priority:** Low-Medium  
**Estimated Effort:** 2-3 hours

## Current State

**Already Implemented:**
- ✅ `abs`, `sign`, `floor`, `ceil`, `fract`, `mod`, `min`, `max`, `clamp`, `mix`, `step`, `smoothstep`

**Not Yet Implemented:**
- ❌ `trunc`, `round`, `roundEven`
- ❌ `modf`, `isnan`, `isinf`
- ❌ `faceforward`, `reflect`, `refract`

## Requirements

### Common Functions

**Rounding Functions:**
- `trunc(genFType x)` - Truncate to integer (toward zero)
- `round(genFType x)` - Round to nearest integer
- `roundEven(genFType x)` - Round to nearest even integer

**Special Functions:**
- `modf(genFType x, out genFType i)` - Separate integer and fractional parts
- `isnan(genFType x)` - Test for NaN (returns bool/bvec)
- `isinf(genFType x)` - Test for infinity (returns bool/bvec)

### Geometric Functions

**Vector Operations:**
- `faceforward(genFType N, genFType I, genFType Nref)` - Orient normal
- `reflect(genFType I, genFType N)` - Reflection direction
- `refract(genFType I, genFType N, float eta)` - Refraction direction

## Implementation Strategy

### 1. Builtin Registration (`semantic/builtins.rs`)

Add signatures for all functions:

```rust
"trunc" => Some(vec![BuiltinSignature {
    name: "trunc",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"round" => Some(vec![BuiltinSignature {
    name: "round",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"roundEven" => Some(vec![BuiltinSignature {
    name: "roundEven",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"isnan" => Some(vec![BuiltinSignature {
    name: "isnan",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0), // Returns bool/bvec
}]),

"isinf" => Some(vec![BuiltinSignature {
    name: "isinf",
    param_types: vec![BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0), // Returns bool/bvec
}]),

"faceforward" => Some(vec![BuiltinSignature {
    name: "faceforward",
    param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType, BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"reflect" => Some(vec![BuiltinSignature {
    name: "reflect",
    param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),

"refract" => Some(vec![BuiltinSignature {
    name: "refract",
    param_types: vec![BuiltinParamType::GenFType, BuiltinParamType::GenFType, BuiltinParamType::Float],
    return_type: BuiltinReturnType::SameAsParam(0),
}]),
```

### 2. Code Generation (`codegen/builtins.rs`)

**Rounding Functions:**

```rust
fn builtin_trunc(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();
    
    for &val in x_vals {
        // trunc(x) = sign(x) * floor(abs(x))
        let abs_val = self.builder.ins().fabs(val);
        let floored = self.builder.ins().floor(abs_val);
        let sign = self.builtin_sign(vec![(vec![val], x_ty.clone())])?.0[0];
        result_vals.push(self.builder.ins().fmul(sign, floored));
    }
    
    Ok((result_vals, x_ty.clone()))
}

fn builtin_round(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();
    
    for &val in x_vals {
        // round(x) = floor(x + 0.5) for positive, floor(x - 0.5) for negative
        let half = self.builder.ins().f32const(0.5);
        let zero = self.builder.ins().f32const(0.0);
        let is_neg = self.builder.ins().fcmp(FloatCC::LessThan, val, zero);
        let offset = self.builder.ins().select(is_neg, 
            self.builder.ins().fsub(zero, half), half);
        let adjusted = self.builder.ins().fadd(val, offset);
        result_vals.push(self.builder.ins().floor(adjusted));
    }
    
    Ok((result_vals, x_ty.clone()))
}

fn builtin_roundEven(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    // More complex: round to nearest even integer
    // For now, can use libcall or approximate with round()
    // Full implementation requires checking if fractional part is exactly 0.5
    // and rounding toward even integer
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();
    
    for &val in x_vals {
        // Simplified: use round() for now, full implementation later
        let rounded = self.builtin_round(vec![(vec![val], x_ty.clone())])?.0[0];
        result_vals.push(rounded);
    }
    
    Ok((result_vals, x_ty.clone()))
}
```

**Special Functions:**

```rust
fn builtin_isnan(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();
    
    for &val in x_vals {
        // isnan(x) = x != x (NaN is the only value not equal to itself)
        let cmp = self.builder.ins().fcmp(FloatCC::Unordered, val, val);
        result_vals.push(cmp);
    }
    
    // Return type is bool/bvec (same component count)
    let result_ty = if x_ty.is_vector() {
        Type::BVec2 // or BVec3/BVec4 based on component count
    } else {
        Type::Bool
    };
    
    Ok((result_vals, result_ty))
}

fn builtin_isinf(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (x_vals, x_ty) = &args[0];
    let mut result_vals = Vec::new();
    
    for &val in x_vals {
        // isinf(x) = abs(x) == infinity
        let abs_val = self.builder.ins().fabs(val);
        let inf = self.builder.ins().f32const(f32::INFINITY);
        let cmp = self.builder.ins().fcmp(FloatCC::Equal, abs_val, inf);
        result_vals.push(cmp);
    }
    
    let result_ty = if x_ty.is_vector() {
        Type::BVec2 // or BVec3/BVec4
    } else {
        Type::Bool
    };
    
    Ok((result_vals, result_ty))
}
```

**Geometric Functions:**

```rust
fn builtin_faceforward(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (n_vals, n_ty) = &args[0];
    let (i_vals, _) = &args[1];
    let (nref_vals, _) = &args[2];
    
    // faceforward(N, I, Nref) = dot(Nref, I) < 0 ? N : -N
    let dot_result = self.builtin_dot(vec![
        (nref_vals.clone(), n_ty.clone()),
        (i_vals.clone(), n_ty.clone()),
    ])?;
    let dot_val = dot_result.0[0];
    
    let zero = self.builder.ins().f32const(0.0);
    let is_negative = self.builder.ins().fcmp(FloatCC::LessThan, dot_val, zero);
    
    let mut result_vals = Vec::new();
    for (i, &n_val) in n_vals.iter().enumerate() {
        let neg_n = self.builder.ins().fsub(zero, n_val);
        result_vals.push(self.builder.ins().select(is_negative, n_val, neg_n));
    }
    
    Ok((result_vals, n_ty.clone()))
}

fn builtin_reflect(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (i_vals, i_ty) = &args[0];
    let (n_vals, _) = &args[1];
    
    // reflect(I, N) = I - 2 * dot(N, I) * N
    let dot_ni = self.builtin_dot(vec![
        (n_vals.clone(), i_ty.clone()),
        (i_vals.clone(), i_ty.clone()),
    ])?;
    let dot_val = dot_ni.0[0];
    
    let two = self.builder.ins().f32const(2.0);
    let two_dot = self.builder.ins().fmul(two, dot_val);
    
    let mut result_vals = Vec::new();
    for (i, &i_val) in i_vals.iter().enumerate() {
        let n_component = n_vals[i];
        let two_dot_n = self.builder.ins().fmul(two_dot, n_component);
        result_vals.push(self.builder.ins().fsub(i_val, two_dot_n));
    }
    
    Ok((result_vals, i_ty.clone()))
}

fn builtin_refract(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (i_vals, i_ty) = &args[0];
    let (n_vals, _) = &args[1];
    let (eta_vals, _) = &args[2];
    let eta = eta_vals[0];
    
    // k = 1.0 - eta * eta * (1.0 - dot(N,I) * dot(N,I))
    let dot_ni = self.builtin_dot(vec![
        (n_vals.clone(), i_ty.clone()),
        (i_vals.clone(), i_ty.clone()),
    ])?;
    let dot_val = dot_ni.0[0];
    let dot_sq = self.builder.ins().fmul(dot_val, dot_val);
    
    let one = self.builder.ins().f32const(1.0);
    let one_minus_dot_sq = self.builder.ins().fsub(one, dot_sq);
    let eta_sq = self.builder.ins().fmul(eta, eta);
    let eta_sq_times = self.builder.ins().fmul(eta_sq, one_minus_dot_sq);
    let k = self.builder.ins().fsub(one, eta_sq_times);
    
    // if k < 0.0, return 0, else return eta * I - (eta * dot(N,I) + sqrt(k)) * N
    let zero = self.builder.ins().f32const(0.0);
    let k_negative = self.builder.ins().fcmp(FloatCC::LessThan, k, zero);
    
    let sqrt_k = self.builder.ins().sqrt(k);
    let eta_dot = self.builder.ins().fmul(eta, dot_val);
    let eta_dot_plus_sqrt = self.builder.ins().fadd(eta_dot, sqrt_k);
    
    let mut result_vals = Vec::new();
    for (i, &i_val) in i_vals.iter().enumerate() {
        let n_component = n_vals[i];
        let eta_i = self.builder.ins().fmul(eta, i_val);
        let second_term = self.builder.ins().fmul(eta_dot_plus_sqrt, n_component);
        let refracted = self.builder.ins().fsub(eta_i, second_term);
        result_vals.push(self.builder.ins().select(k_negative, zero, refracted));
    }
    
    Ok((result_vals, i_ty.clone()))
}
```

## Testing Strategy

### Functionality Tests

**Location:** `crates/lp-glsl-filetests/filetests/additional_builtins/`

**Rounding:**
```glsl
// Test: trunc_scalar.glsl
// Spec: builtinfunctions.adoc:401-415
float main() {
    return trunc(3.7);
}
// run: == 3.0

// Test: round_scalar.glsl
// Spec: builtinfunctions.adoc:416-434
float main() {
    return round(3.5);
}
// run: == 4.0 (or 3.0, implementation-defined)
```

**Geometric:**
```glsl
// Test: reflect_vec3.glsl
// Spec: builtinfunctions.adoc:1073-1084
vec3 main() {
    vec3 I = vec3(1.0, -1.0, 0.0);
    vec3 N = vec3(0.0, 1.0, 0.0);
    return reflect(I, N);
}
// run: == vec3(1.0, 1.0, 0.0)
```

### Error Handling Tests

```glsl
// Test: builtin_wrong_arg_type.glsl
float main() {
    return trunc(true);  // ERROR
}
// EXPECT_ERROR: No matching overload for trunc([Bool])
```

## Success Criteria

- [ ] All 8 additional functions implemented
- [ ] Functions registered in `semantic/builtins.rs`
- [ ] Code generation in `codegen/builtins.rs`
- [ ] Minimum 10 functionality tests pass
- [ ] Minimum 3 error handling tests pass
- [ ] Code follows existing patterns
- [ ] No regressions

## Notes

- `roundEven` can be simplified initially
- `modf` requires out parameters (may need special handling)
- `isnan`/`isinf` return bool/bvec (different return type pattern)

