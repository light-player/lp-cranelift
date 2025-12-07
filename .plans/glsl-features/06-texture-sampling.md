# Texture Sampling Implementation

## Overview

Implement texture access functions for runtime integration. This requires defining the ABI between the compiler and runtime system.

**Spec Reference:** `builtinfunctions.adoc` lines 1751-2300 (Texture Functions)  
**Priority:** Low (requires runtime)  
**Estimated Effort:** 2-3 hours

## Current State

- ❌ No sampler types recognized
- ❌ No texture functions implemented
- ❌ No runtime ABI defined

## Requirements

### Sampler Types

```glsl
uniform sampler2D tex;
uniform samplerCube cubemap;
```

**Types:**
- `sampler2D`, `sampler3D`, `samplerCube`
- `isampler2D`, `usampler2D` (integer textures)
- Opaque handles (cannot be directly accessed)

### Texture Functions

**Basic Lookup:**
- `texture(sampler2D, vec2)` - Basic 2D texture lookup
- `texture(samplerCube, vec3)` - Cubemap lookup

**Query Functions:**
- `textureSize(sampler, int lod)` - Get texture dimensions

**Fetch Functions:**
- `texelFetch(sampler, ivec2, int lod)` - Fetch specific texel (no filtering)

## Implementation Strategy

### 1. Type System (`semantic/types.rs`)

Add sampler types:

```rust
impl Type {
    pub fn is_sampler(&self) -> bool {
        matches!(self,
            Type::Sampler2D | Type::Sampler3D | Type::SamplerCube |
            Type::ISampler2D | Type::USampler2D
        )
    }
}
```

### 2. Semantic Analysis

**Sampler validation:**
- Samplers can only be declared as `uniform` (for now)
- Cannot be assigned or used in expressions (except as function arguments)

### 3. Code Generation

**External function calls:**

```rust
// In codegen/builtins.rs
fn builtin_texture(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
    let (sampler_vals, sampler_ty) = &args[0];
    let (coord_vals, _) = &args[1];
    
    // Declare external function
    let sig = self.module.signatures.get(self.texture_sig)?;
    let func_ref = self.module.declare_function("__glsl_texture2D", Linkage::Import, sig)?;
    
    // Call with sampler handle and coordinates
    let call_inst = self.builder.ins().call(func_ref, &[sampler_vals[0], coord_vals[0], coord_vals[1]]);
    
    // Returns vec4 (RGBA)
    Ok((vec![call_inst], Type::Vec4))
}
```

### 4. Runtime ABI

Define C ABI for texture functions:

```c
// Runtime interface
typedef void* glsl_sampler2d_t;

vec4 glsl_texture2D(glsl_sampler2d_t sampler, float u, float v);
ivec2 glsl_textureSize(glsl_sampler2d_t sampler, int lod);
vec4 glsl_texelFetch(glsl_sampler2d_t sampler, int x, int y, int lod);
```

## Testing Strategy

### Compilation Tests

**Location:** `crates/lp-glsl-filetests/filetests/textures/`

```glsl
// Test: texture2D_basic.glsl
// Spec: builtinfunctions.adoc:1751-1791
uniform sampler2D tex;

vec4 main() {
    vec2 uv = vec2(0.5, 0.5);
    return texture(tex, uv);
}
// CHECK: call to __glsl_texture2D
// Note: Runtime execution requires texture data
```

### Error Handling Tests

```glsl
// Test: texture_non_uniform.glsl
sampler2D tex;  // ERROR: must be uniform
// EXPECT_ERROR: sampler types must be uniform

// Test: texture_wrong_arg_type.glsl
uniform sampler2D tex;
vec4 main() {
    return texture(tex, 1.0);  // ERROR: needs vec2
}
// EXPECT_ERROR: texture() requires vec2 coordinates
```

## Success Criteria

- [ ] Sampler types recognized
- [ ] Texture functions generate external calls
- [ ] ABI defined for runtime integration
- [ ] Minimum 3 compilation tests pass
- [ ] Minimum 2 error handling tests pass

## Future Enhancements

- Full texture function set (LOD, offset, gradients)
- Cubemap support
- Integer texture support
- Runtime implementation

## Notes

- This is primarily an ABI definition phase
- Actual texture sampling done by runtime
- Can defer full implementation until runtime is ready

