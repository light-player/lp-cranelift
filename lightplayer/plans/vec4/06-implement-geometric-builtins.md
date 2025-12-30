# Implement Geometric Builtin Functions

## Problem

Geometric builtin functions `faceforward`, `reflect`, and `refract` are not implemented.

**Missing functions:**
- `faceforward(vec4, vec4, vec4)` → `vec4` - face forward vector
- `reflect(vec4, vec4)` → `vec4` - reflection vector
- `refract(vec4, vec4, float)` → `vec4` - refraction vector

**Affected tests:**
- `vec4/builtins/faceforward.glsl:19` - `test_vec4_faceforward_positive_dot()` fails
- `vec4/builtins/reflect.glsl:19` - `test_vec4_reflect_simple()` fails
- `vec4/builtins/refract.glsl:19` - `test_vec4_refract_normal_incidence()` fails

## Root Cause

These functions are not registered in the builtin function lookup and don't have codegen implementations.

## Fix Strategy

1. **Add function signatures** to `semantic/builtins.rs`:
   - Add signatures for `faceforward`, `reflect`, `refract`
   - Define parameter types and return types

2. **Implement codegen** in `codegen/builtins/geometric.rs`:
   - Implement each function according to GLSL spec:
     - `faceforward(N, I, Nref) = dot(Nref, I) < 0 ? N : -N`
     - `reflect(I, N) = I - 2 * dot(N, I) * N`
     - `refract(I, N, eta)` - more complex, see GLSL spec

3. **Register functions** in `codegen/builtins/mod.rs`:
   - Add cases to `translate_builtin_call` match statement

## Implementation Steps

1. **Add signatures** in `semantic/builtins.rs`:
   ```rust
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

2. **Implement in `codegen/builtins/geometric.rs`**:

   **faceforward:**
   ```rust
   pub fn builtin_faceforward(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
       let (n_vals, n_ty) = &args[0];
       let (i_vals, _) = &args[1];
       let (nref_vals, _) = &args[2];
       
       // Compute dot(Nref, I)
       let dot_val = self.builtin_dot(vec![(nref_vals.clone(), Type::Vec4), (i_vals.clone(), Type::Vec4)])?.0[0];
       
       // Check if dot < 0
       let zero = self.builder.ins().f32const(0.0);
       let is_negative = self.builder.ins().fcmp(types::F32, FloatCC::LessThan, dot_val, zero);
       
       // Return N if dot < 0, else -N
       let neg_n = n_vals.iter().map(|&v| self.builder.ins().fneg(v)).collect();
       // Use select to choose between N and -N
       // ... implementation
   }
   ```

   **reflect:**
   ```rust
   pub fn builtin_reflect(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
       let (i_vals, i_ty) = &args[0];
       let (n_vals, _) = &args[1];
       
       // Compute dot(N, I)
       let dot_val = self.builtin_dot(vec![(n_vals.clone(), Type::Vec4), (i_vals.clone(), Type::Vec4)])?.0[0];
       
       // Compute 2 * dot(N, I) * N
       let two = self.builder.ins().f32const(2.0);
       let two_dot = self.builder.ins().fmul(two, dot_val);
       let scaled_n: Vec<Value> = n_vals.iter().map(|&v| self.builder.ins().fmul(two_dot, v)).collect();
       
       // Compute I - 2 * dot(N, I) * N
       let result: Vec<Value> = i_vals.iter().zip(scaled_n.iter())
           .map(|(&i, &s)| self.builder.ins().fsub(i, s))
           .collect();
       
       Ok((result, i_ty.clone()))
   }
   ```

   **refract:**
   - More complex, see GLSL spec formula
   - `k = 1.0 - eta * eta * (1.0 - dot(N, I) * dot(N, I))`
   - If `k < 0.0`, return zero vector
   - Otherwise: `eta * I - (eta * dot(N, I) + sqrt(k)) * N`

3. **Register in `codegen/builtins/mod.rs`**:
   ```rust
   "faceforward" => self.builtin_faceforward(args),
   "reflect" => self.builtin_reflect(args),
   "refract" => self.builtin_refract(args),
   ```

## Files to Modify

- `lightplayer/crates/lp-glsl/src/semantic/builtins.rs` - Add function signatures
- `lightplayer/crates/lp-glsl/src/codegen/builtins/mod.rs` - Register functions
- `lightplayer/crates/lp-glsl/src/codegen/builtins/geometric.rs` - Add implementations

## Test Cases

- `vec4/builtins/faceforward.glsl` - All tests should pass
- `vec4/builtins/reflect.glsl` - All tests should pass
- `vec4/builtins/refract.glsl` - All tests should pass

## Acceptance Criteria

- [ ] All geometric builtin function tests pass
- [ ] Functions work for vec2, vec3, vec4 (if tests exist)
- [ ] No regressions in other tests
- [ ] Code compiles without warnings

## Verification

Run the tests:
```bash
scripts/glsl-filetests.sh vec4/builtins/faceforward.glsl
scripts/glsl-filetests.sh vec4/builtins/reflect.glsl
scripts/glsl-filetests.sh vec4/builtins/refract.glsl
```

Expected result: All tests pass.

## Commit Instructions

Once all tests pass:

```bash
git add -A
git commit -m "lpc: implement geometric builtin functions faceforward, reflect, refract"
```





