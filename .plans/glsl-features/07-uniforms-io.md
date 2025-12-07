# Uniforms and Shader I/O Implementation

## Overview

Implement storage qualifiers (uniform, in, out) and shader input/output variables. Essential for real-world shader applications.

**Spec Reference:** `variables.adoc` lines 1230-1550 (Storage Qualifiers), 2109-2648 (Input/Output Variables)  
**Priority:** Medium  
**Estimated Effort:** 3-4 hours

## Current State

- ❌ No storage qualifiers recognized
- ❌ No uniform variable support
- ❌ No input/output variable support
- ❌ No built-in variables (gl_Position, gl_FragColor)

## Requirements

### Storage Qualifiers

**Uniform:**
```glsl
uniform float time;
uniform vec3 lightPosition;
```

**Input/Output:**
```glsl
in vec3 position;
out vec4 fragColor;
```

**Const:**
```glsl
const float PI = 3.14159;
```

### Built-in Variables

**Vertex Shader Output:**
- `gl_Position` (vec4) - Clip space position

**Fragment Shader:**
- `gl_FragColor` (vec4) - Output color
- `gl_FragCoord` (vec4) - Fragment coordinates (input)

### Behavior

- Uniforms: Read-only, passed from application
- Inputs: Read-only, from previous shader stage
- Outputs: Write-only, to next shader stage
- Const: Compile-time constant

## Implementation Strategy

### 1. Semantic Analysis

**Track storage qualifiers:**

```rust
// In semantic/scope.rs or new semantic/qualifiers.rs
pub enum StorageQualifier {
    Uniform,
    In,
    Out,
    Const,
    None,
}

pub struct VariableInfo {
    pub name: String,
    pub ty: Type,
    pub qualifier: StorageQualifier,
    // ...
}
```

**Validate qualifier usage:**
- Uniforms cannot be assigned (except at declaration)
- Inputs cannot be assigned
- Outputs can be assigned
- Const must have initializer

### 2. Code Generation

**Uniforms as function parameters:**

```rust
// In codegen/signature.rs
fn build_function_signature_with_uniforms(
    func: &TypedFunction,
    uniforms: &[UniformVariable],
) -> Signature {
    let mut sig = Signature::new(CallConv::SystemV);
    
    // Add uniform parameters
    for uniform in uniforms {
        let param_ty = uniform.ty.to_cranelift_type();
        sig.params.push(AbiParam::new(param_ty));
    }
    
    // Add regular parameters
    // ...
    
    sig
}
```

**Built-in variables:**

```rust
// In codegen/context.rs
pub fn translate_builtin_variable(
    &mut self,
    name: &str,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    match name {
        "gl_FragCoord" => {
            // Pass as function parameter
            let param = self.function_params[self.frag_coord_param_index];
            Ok((vec![param], GlslType::Vec4))
        }
        _ => Err(GlslError::new(ErrorCode::E0400, format!("unknown built-in: {}", name))),
    }
}
```

### 3. ABI Definition

**Shader entry point signature:**

```rust
// For fragment shader with uniforms
fn fragment_shader_main(
    gl_FragCoord: vec4,      // Built-in input
    uniform_time: float,      // Uniform
    uniform_lightPos: vec3,  // Uniform
) -> vec4 {
    // ...
}
```

## Testing Strategy

### Functionality Tests

**Location:** `crates/lp-glsl-filetests/filetests/io/`

```glsl
// Test: uniform_declaration.glsl
// Spec: variables.adoc:2381-2464
uniform float time;

float main() {
    return time;
}
// CHECK: time passed as parameter
```

```glsl
// Test: in_out_variables.glsl
// Spec: variables.adoc:2109-2379, 2465-2648
in vec3 position;
out vec4 fragColor;

vec4 main() {
    fragColor = vec4(position, 1.0);
    return fragColor;
}
// CHECK: input parameter, output written
```

### Error Handling Tests

```glsl
// Test: uniform_assignment.glsl
uniform float time;
float main() {
    time = 5.0;  // ERROR: cannot assign uniform
}
// EXPECT_ERROR: uniform variables cannot be assigned

// Test: in_assignment.glsl
in vec3 position;
vec3 main() {
    position = vec3(1.0);  // ERROR: cannot assign input
}
// EXPECT_ERROR: input variables cannot be assigned
```

## Success Criteria

- [ ] Uniform qualifier recognized and validated
- [ ] In/out qualifiers work
- [ ] Storage qualifiers prevent invalid operations
- [ ] Built-in variables supported
- [ ] Minimum 5 functionality tests pass
- [ ] Minimum 3 error handling tests pass

## Future Enhancements

- Uniform blocks
- Layout qualifiers
- Interface matching
- Multiple shader stages

## Notes

- Start with simple uniforms (scalars, vectors)
- Built-ins can be special-cased initially
- ABI design should be extensible for future features

