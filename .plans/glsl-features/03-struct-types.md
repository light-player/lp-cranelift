# Struct Types Implementation

## Overview

Implement user-defined struct types with member declarations, constructors, and member access. Structs enable complex data structures essential for realistic shader programming.

**Spec Reference:** `variables.adoc` lines 814-914, `operators.adoc` lines 298-326  
**Priority:** Medium  
**Estimated Effort:** 4-6 hours

## Current State

- ✅ `Type::Struct(StructId)` exists in type system
- ❌ No semantic analysis for struct declarations
- ❌ No codegen for struct construction
- ❌ No codegen for member access
- ❌ No storage layout calculation

## Requirements

### Struct Declaration

```glsl
struct Light {
    vec3 position;
    vec3 color;
    float intensity;
};

Light myLight;  // Variable declaration
```

**Requirements:**
- Struct name becomes a new type
- Must have at least one member
- Members can be any type (scalars, vectors, matrices, arrays, other structs)
- Members cannot have initializers in declaration
- Anonymous structs not supported
- Embedded struct definitions not supported

### Struct Construction

```glsl
Light myLight = Light(vec3(1.0, 2.0, 3.0), vec3(1.0, 0.5, 0.3), 0.8);
```

**Requirements:**
- Constructor takes arguments in member declaration order
- All members must be provided (or use default initialization)
- Type of each argument must match member type (with implicit conversion)

### Member Access

```glsl
vec3 pos = myLight.position;
myLight.intensity = 1.5;
```

**Requirements:**
- Dot notation for member access
- Members are l-values (can be assigned)
- Nested struct access: `outer.inner.field`

### Memory Layout

- Follow GLSL alignment rules (std140/std430 style)
- Calculate field offsets
- Handle padding between fields
- Whole struct alignment

## Implementation Strategy

### 1. Type System (`semantic/types.rs`)

**Current state:** `Type::Struct(StructId)` exists, but no struct definitions stored.

**Add struct definition storage:**

```rust
// In semantic/types.rs or new semantic/structs.rs
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
    pub id: StructId,
}

pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub offset: usize,  // Calculated during layout
}

pub struct StructRegistry {
    structs: Vec<StructDefinition>,
    name_to_id: HashMap<String, StructId>,
}

impl StructRegistry {
    pub fn register(&mut self, def: StructDefinition) -> StructId;
    pub fn lookup(&self, name: &str) -> Option<StructId>;
    pub fn get(&self, id: StructId) -> &StructDefinition;
}
```

### 2. Semantic Analysis (`semantic/passes/`)

**New pass:** `struct_collection.rs`

```rust
pub struct StructCollectionPass {
    registry: StructRegistry,
}

impl SemanticPass for StructCollectionPass {
    fn run(&mut self, shader: &TranslationUnit, source: &str) -> Result<(), GlslError> {
        // Traverse AST to find struct declarations
        // Parse struct name and members
        // Register in StructRegistry
        // Calculate memory layout
    }
}
```

**Parse struct declarations:**

```rust
fn parse_struct_declaration(
    decl: &glsl::syntax::StructSpecifier,
) -> Result<StructDefinition, GlslError> {
    let name = decl.name.as_ref()
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, "anonymous structs not supported"))?;
    
    if decl.fields.is_empty() {
        return Err(GlslError::new(ErrorCode::E0400, "struct must have at least one member"));
    }
    
    let mut fields = Vec::new();
    for field in &decl.fields {
        // Parse field type and names
        let field_ty = parse_type_specifier(&field.ty)?;
        for declarator in &field.identifiers {
            fields.push(StructField {
                name: declarator.name.clone(),
                ty: field_ty.clone(),
                offset: 0, // Calculate later
            });
        }
    }
    
    // Calculate memory layout
    calculate_struct_layout(&mut fields);
    
    Ok(StructDefinition {
        name: name.name.clone(),
        fields,
        id: 0, // Assign during registration
    })
}
```

**Memory layout calculation:**

```rust
fn calculate_struct_layout(fields: &mut [StructField]) {
    let mut offset = 0;
    
    for field in fields.iter_mut() {
        // Align offset to field type alignment
        let align = field.ty.alignment();
        offset = (offset + align - 1) / align * align;
        
        field.offset = offset;
        offset += field.ty.size();
    }
}

impl Type {
    fn alignment(&self) -> usize {
        match self {
            Type::Float | Type::Int => 4,
            Type::Vec2 => 8,
            Type::Vec3 | Type::Vec4 => 16,
            Type::Mat2 => 8,
            Type::Mat3 => 16,
            Type::Mat4 => 16,
            Type::Struct(id) => {
                // Get struct definition and return its alignment
                // (largest member alignment)
            }
            _ => 1,
        }
    }
    
    fn size(&self) -> usize {
        match self {
            Type::Float | Type::Int => 4,
            Type::Vec2 => 8,
            Type::Vec3 => 12,
            Type::Vec4 => 16,
            Type::Mat2 => 16,  // 4 floats
            Type::Mat3 => 36,  // 9 floats
            Type::Mat4 => 64,  // 16 floats
            Type::Struct(id) => {
                // Sum of all field sizes + padding
            }
            _ => 0,
        }
    }
}
```

### 3. Type Checking (`semantic/type_check/`)

**Add struct constructor checking:**

```rust
// In semantic/type_check/constructors.rs
pub fn check_struct_constructor(
    struct_name: &str,
    args: &[&Expr],
    struct_registry: &StructRegistry,
    symbols: &SymbolTable,
) -> Result<Type, GlslError> {
    let struct_id = struct_registry.lookup(struct_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("unknown struct type: {}", struct_name)))?;
    
    let struct_def = struct_registry.get(struct_id);
    
    if args.len() != struct_def.fields.len() {
        return Err(GlslError::new(ErrorCode::E0104,
            format!("struct constructor expects {} arguments, got {}", struct_def.fields.len(), args.len())));
    }
    
    // Check each argument type matches field type
    for (i, (arg, field)) in args.iter().zip(struct_def.fields.iter()).enumerate() {
        let arg_ty = infer_expr_type_with_registry(arg, symbols, None)?;
        if !can_implicitly_convert(&arg_ty, &field.ty) {
            return Err(GlslError::new(ErrorCode::E0106,
                format!("struct constructor argument {}: expected {:?}, got {:?}", i, field.ty, arg_ty)));
        }
    }
    
    Ok(Type::Struct(struct_id))
}
```

**Add member access checking:**

```rust
// In semantic/type_check/inference.rs
fn infer_member_access(
    base_expr: &Expr,
    member_name: &str,
    symbols: &SymbolTable,
    struct_registry: &StructRegistry,
) -> Result<Type, GlslError> {
    let base_ty = infer_expr_type_with_registry(base_expr, symbols, None)?;
    
    if let Type::Struct(struct_id) = base_ty {
        let struct_def = struct_registry.get(struct_id);
        let field = struct_def.fields.iter()
            .find(|f| f.name == member_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400,
                format!("struct {} has no member '{}'", struct_def.name, member_name)))?;
        
        Ok(field.ty.clone())
    } else {
        Err(GlslError::new(ErrorCode::E0106,
            format!("member access requires struct type, got {:?}", base_ty)))
    }
}
```

### 4. Code Generation (`codegen/`)

**Struct storage:**

Structs are stored on the stack as consecutive memory. Use `VirtualValue::StackSlot` with calculated size.

```rust
// In codegen/context.rs
pub struct StructValue {
    pub struct_id: StructId,
    pub base_addr: Value,  // Pointer to struct on stack
    pub size: usize,
}
```

**Struct construction:**

```rust
// In codegen/expr/constructor.rs
pub fn translate_struct_constructor(
    ctx: &mut CodegenContext,
    struct_name: &str,
    args: &[Expr],
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let struct_id = ctx.struct_registry.lookup(struct_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("unknown struct: {}", struct_name)))?;
    
    let struct_def = ctx.struct_registry.get(struct_id);
    let struct_size = struct_def.size();
    
    // Allocate stack space for struct
    let struct_ptr = ctx.allocate_stack_slot(struct_size);
    
    // Evaluate each argument and store at field offset
    for (i, (arg_expr, field)) in args.iter().zip(struct_def.fields.iter()).enumerate() {
        let (arg_vals, arg_ty) = ctx.translate_expr_typed(arg_expr)?;
        
        // Coerce to field type if needed
        let coerced_vals = coerce_to_type(ctx, arg_vals, &arg_ty, &field.ty)?;
        
        // Store at field offset
        store_at_offset(ctx, struct_ptr, field.offset, coerced_vals, &field.ty)?;
    }
    
    // Return pointer to struct (or values if small enough)
    Ok((vec![struct_ptr], GlslType::Struct(struct_id)))
}
```

**Member access:**

```rust
// In codegen/expr/component.rs (extend existing)
pub fn translate_struct_member_access(
    ctx: &mut CodegenContext,
    base_expr: &Expr,
    member_name: &str,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let (base_vals, base_ty) = ctx.translate_expr_typed(base_expr)?;
    
    if let GlslType::Struct(struct_id) = base_ty {
        let struct_def = ctx.struct_registry.get(struct_id);
        let field = struct_def.fields.iter()
            .find(|f| f.name == member_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("no member '{}'", member_name)))?;
        
        // base_vals[0] is pointer to struct
        let struct_ptr = base_vals[0];
        
        // Load field at offset
        let field_vals = load_at_offset(ctx, struct_ptr, field.offset, &field.ty)?;
        
        Ok((field_vals, field.ty.clone()))
    } else {
        Err(GlslError::new(ErrorCode::E0106, "member access requires struct"))
    }
}
```

**Member assignment:**

```rust
// In codegen/stmt.rs (extend assignment handling)
fn translate_struct_member_assignment(
    ctx: &mut CodegenContext,
    base_expr: &Expr,
    member_name: &str,
    rhs: &Expr,
) -> Result<(), GlslError> {
    let (base_vals, base_ty) = ctx.translate_expr_typed(base_expr)?;
    
    if let GlslType::Struct(struct_id) = base_ty {
        let struct_def = ctx.struct_registry.get(struct_id);
        let field = struct_def.fields.iter()
            .find(|f| f.name == member_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("no member '{}'", member_name)))?;
        
        let (rhs_vals, rhs_ty) = ctx.translate_expr_typed(rhs)?;
        let coerced_vals = coerce_to_type(ctx, rhs_vals, &rhs_ty, &field.ty)?;
        
        let struct_ptr = base_vals[0];
        store_at_offset(ctx, struct_ptr, field.offset, coerced_vals, &field.ty)?;
        
        Ok(())
    } else {
        Err(GlslError::new(ErrorCode::E0106, "member assignment requires struct"))
    }
}
```

### 5. JIT Calling Conventions (`codegen/`)

**Struct return values in function signatures:**

Structs must be expanded into multiple return values (one per field, recursively), similar to how matrices are handled.

```rust
// In codegen/signature.rs - extend add_type_as_returns()
fn add_type_as_returns(sig: &mut Signature, ty: &Type, struct_registry: &StructRegistry) {
    if ty.is_vector() {
        // Vector: return each component
        let base_ty = ty.vector_base_type().unwrap();
        let cranelift_ty = base_ty.to_cranelift_type();
        let count = ty.component_count().unwrap();
        for _ in 0..count {
            sig.returns.push(AbiParam::new(cranelift_ty));
        }
    } else if ty.is_matrix() {
        // Matrix: return each element (column-major)
        let element_count = ty.matrix_element_count().unwrap();
        let cranelift_ty = Type::Float.to_cranelift_type();
        for _ in 0..element_count {
            sig.returns.push(AbiParam::new(cranelift_ty));
        }
    } else if let Type::Struct(struct_id) = ty {
        // Struct: return each field recursively
        let struct_def = struct_registry.get(*struct_id);
        for field in &struct_def.fields {
            Self::add_type_as_returns(sig, &field.ty, struct_registry);
        }
    } else if let Type::Array(elem_ty, size) = ty {
        // Array: return each element recursively
        for _ in 0..*size {
            Self::add_type_as_returns(sig, elem_ty, struct_registry);
        }
    } else {
        // Scalar: single return value
        let cranelift_ty = ty.to_cranelift_type();
        sig.returns.push(AbiParam::new(cranelift_ty));
    }
}

// Also update count_returns() similarly
pub fn count_returns(ty: &Type, struct_registry: &StructRegistry) -> usize {
    if ty == &Type::Void {
        0
    } else if ty.is_vector() {
        ty.component_count().unwrap()
    } else if ty.is_matrix() {
        ty.matrix_element_count().unwrap()
    } else if let Type::Struct(struct_id) = ty {
        let struct_def = struct_registry.get(*struct_id);
        struct_def.fields.iter()
            .map(|f| Self::count_returns(&f.ty, struct_registry))
            .sum()
    } else if let Type::Array(elem_ty, size) = ty {
        Self::count_returns(elem_ty, struct_registry) * size
    } else {
        1
    }
}
```

**Struct return statement handling:**

```rust
// In codegen/stmt.rs - extend return statement handling
if expected_ty.is_vector() || expected_ty.is_matrix() {
    // ... existing vector/matrix handling ...
} else if expected_ty.is_struct() || expected_ty.is_array() {
    // For structs/arrays, flatten all values recursively
    let flattened_vals = flatten_struct_or_array_values(
        ctx, ret_vals, &ret_ty, expected_ty, struct_registry
    )?;
    self.builder.ins().return_(&flattened_vals);
} else {
    // ... existing scalar handling ...
}

// Helper function to flatten struct/array values
fn flatten_struct_or_array_values(
    ctx: &mut CodegenContext,
    values: Vec<Value>,
    ret_ty: &GlslType,
    expected_ty: &GlslType,
    struct_registry: &StructRegistry,
) -> Result<Vec<Value>, GlslError> {
    match (ret_ty, expected_ty) {
        (GlslType::Struct(ret_id), GlslType::Struct(expected_id)) => {
            if ret_id != expected_id {
                return Err(GlslError::new(ErrorCode::E0106,
                    format!("struct type mismatch in return")));
            }
            let struct_def = struct_registry.get(*ret_id);
            // values[0] is pointer to struct
            let struct_ptr = values[0];
            let mut flattened = Vec::new();
            
            for field in &struct_def.fields {
                let field_vals = load_at_offset(ctx, struct_ptr, field.offset, &field.ty)?;
                let field_flattened = flatten_struct_or_array_values(
                    ctx, field_vals, &field.ty, &field.ty, struct_registry
                )?;
                flattened.extend(field_flattened);
            }
            Ok(flattened)
        }
        (GlslType::Array(ret_elem, ret_size), GlslType::Array(expected_elem, expected_size)) => {
            if ret_size != expected_size {
                return Err(GlslError::new(ErrorCode::E0106,
                    format!("array size mismatch in return")));
            }
            // values[0] is pointer to array
            let array_ptr = values[0];
            let elem_size = ret_elem.size();
            let mut flattened = Vec::new();
            
            for i in 0..*ret_size {
                let offset = i * elem_size;
                let elem_vals = load_at_offset(ctx, array_ptr, offset, ret_elem)?;
                let elem_flattened = flatten_struct_or_array_values(
                    ctx, elem_vals, ret_elem, expected_elem, struct_registry
                )?;
                flattened.extend(elem_flattened);
            }
            Ok(flattened)
        }
        _ => {
            // Already scalar/vector/matrix, return as-is
            Ok(values)
        }
    }
}
```

**Default struct return generation:**

```rust
// In jit.rs - add generate_default_struct_return()
fn generate_default_struct_return(
    ctx: &mut crate::codegen::context::CodegenContext,
    return_type: &crate::semantic::types::Type,
    struct_registry: &crate::semantic::structs::StructRegistry,
) -> Result<(), crate::error::GlslError> {
    use crate::error::{ErrorCode, GlslError};
    
    if let Type::Struct(struct_id) = return_type {
        let struct_def = struct_registry.get(*struct_id);
        let mut vals = Vec::new();
        
        // Generate default value for each field recursively
        for field in &struct_def.fields {
            let field_vals = generate_default_return_value(
                ctx, &field.ty, struct_registry
            )?;
            vals.extend(field_vals);
        }
        
        ctx.builder.ins().return_(&vals);
        Ok(())
    } else {
        Err(GlslError::new(
            ErrorCode::E0400,
            format!("expected struct type, got: {:?}", return_type),
        ))
    }
}

// Helper to generate default return for any type
fn generate_default_return_value(
    ctx: &mut crate::codegen::context::CodegenContext,
    ty: &crate::semantic::types::Type,
    struct_registry: &crate::semantic::structs::StructRegistry,
) -> Result<Vec<cranelift_codegen::ir::Value>, crate::error::GlslError> {
    match ty {
        Type::Float => Ok(vec![ctx.builder.ins().f32const(0.0)]),
        Type::Int => Ok(vec![ctx.builder.ins().iconst(types::I32, 0)]),
        Type::Bool => Ok(vec![ctx.builder.ins().iconst(types::I8, 0)]),
        Type::Struct(struct_id) => {
            let struct_def = struct_registry.get(*struct_id);
            let mut vals = Vec::new();
            for field in &struct_def.fields {
                let field_vals = generate_default_return_value(ctx, &field.ty, struct_registry)?;
                vals.extend(field_vals);
            }
            Ok(vals)
        }
        Type::Array(elem_ty, size) => {
            let mut vals = Vec::new();
            for _ in 0..*size {
                let elem_vals = generate_default_return_value(ctx, elem_ty, struct_registry)?;
                vals.extend(elem_vals);
            }
            Ok(vals)
        }
        _ => {
            // Delegate to existing vector/matrix handlers
            // ...
        }
    }
}
```

**Function call return handling:**

```rust
// In codegen/expr/function.rs - extend return value packaging
// Package return value(s)
if func_sig.return_type == GlslType::Void {
    Ok((vec![], GlslType::Void))
} else if func_sig.return_type.is_vector() {
    let count = func_sig.return_type.component_count().unwrap();
    Ok((return_vals[0..count].to_vec(), func_sig.return_type.clone()))
} else if func_sig.return_type.is_matrix() {
    let count = func_sig.return_type.matrix_element_count().unwrap();
    Ok((return_vals[0..count].to_vec(), func_sig.return_type.clone()))
} else if func_sig.return_type.is_struct() || func_sig.return_type.is_array() {
    // For structs/arrays, all values are already returned flattened
    // We need to reconstruct the pointer representation
    // For now, return all values and let caller handle reconstruction
    // TODO: May need to allocate and store, then return pointer
    let count = SignatureBuilder::count_returns(&func_sig.return_type, struct_registry);
    Ok((return_vals[0..count].to_vec(), func_sig.return_type.clone()))
} else {
    Ok((vec![return_vals[0]], func_sig.return_type.clone()))
}
```

## Testing Strategy

### Functionality Tests

**Location:** `crates/lp-glsl-filetests/filetests/structs/`

**Basic Struct:**
```glsl
// Test: struct_definition.glsl
// Spec: variables.adoc:814-914
struct Point {
    float x;
    float y;
};

float main() {
    Point p = Point(3.0, 4.0);
    return p.x;
}
// run: == 3.0
```

**Member Access:**
```glsl
// Test: struct_member_access.glsl
// Spec: variables.adoc:814-914
struct Color {
    float r;
    float g;
    float b;
};

float main() {
    Color c = Color(1.0, 0.5, 0.0);
    return c.g;
}
// run: == 0.5
```

**Member Assignment:**
```glsl
// Test: struct_member_assign.glsl
// Spec: variables.adoc:814-914
struct Data {
    int value;
};

int main() {
    Data d = Data(10);
    d.value = 20;
    return d.value;
}
// run: == 20
```

**Nested Structs:**
```glsl
// Test: struct_nested.glsl
// Spec: variables.adoc:814-914
struct Inner {
    float x;
};

struct Outer {
    Inner data;
    float y;
};

float main() {
    Outer o = Outer(Inner(5.0), 10.0);
    return o.data.x;
}
// run: == 5.0
```

**Structs with Vectors:**
```glsl
// Test: struct_with_vector.glsl
// Spec: variables.adoc:814-914
struct Light {
    vec3 position;
    vec3 color;
};

vec3 main() {
    Light light = Light(vec3(1.0, 2.0, 3.0), vec3(1.0, 0.5, 0.3));
    return light.color;
}
// run: == vec3(1.0, 0.5, 0.3)
```

**Struct Return from Function:**
```glsl
// Test: struct_return_function.glsl
// Spec: variables.adoc:814-914
struct Point {
    float x;
    float y;
};

Point makePoint(float x, float y) {
    return Point(x, y);
}

Point main() {
    return makePoint(3.0, 4.0);
}
// run: == Point(3.0, 4.0)  // Returns flattened: 3.0, 4.0
```

**Struct Return with Nested:**
```glsl
// Test: struct_return_nested.glsl
// Spec: variables.adoc:814-914
struct Inner {
    float value;
};

struct Outer {
    Inner inner;
    float extra;
};

Outer main() {
    return Outer(Inner(5.0), 10.0);
}
// run: == Outer(Inner(5.0), 10.0)  // Returns flattened: 5.0, 10.0
```

### Error Handling Tests

**Location:** `crates/lp-glsl-filetests/filetests/type_errors/`

```glsl
// Test: struct_unknown_type.glsl
float main() {
    UnknownStruct s = UnknownStruct(1.0);  // ERROR: unknown struct
}
// EXPECT_ERROR: unknown struct type: UnknownStruct

// Test: struct_constructor_wrong_count.glsl
struct Point {
    float x;
    float y;
};

float main() {
    Point p = Point(1.0);  // ERROR: missing argument
}
// EXPECT_ERROR: struct constructor expects 2 arguments, got 1

// Test: struct_constructor_wrong_type.glsl
struct Point {
    float x;
    float y;
};

float main() {
    Point p = Point(1.0, true);  // ERROR: wrong type
}
// EXPECT_ERROR: struct constructor argument 1: expected Float, got Bool

// Test: struct_member_unknown.glsl
struct Point {
    float x;
    float y;
};

float main() {
    Point p = Point(1.0, 2.0);
    return p.z;  // ERROR: no member z
}
// EXPECT_ERROR: struct Point has no member 'z'

// Test: struct_empty.glsl
struct Empty {
};  // ERROR: must have at least one member
// EXPECT_ERROR: struct must have at least one member
```

## Success Criteria

- [ ] Struct declarations parsed and registered
- [ ] Memory layout calculated correctly
- [ ] Struct constructors work
- [ ] Member access generates correct offsets
- [ ] Member assignment works
- [ ] Nested structs supported
- [ ] Structs with vectors/matrices work
- [ ] Struct return values expanded correctly in function signatures
- [ ] Struct return statements flatten values correctly
- [ ] Default struct return generation works
- [ ] Function calls with struct returns work
- [ ] Minimum 8 functionality tests pass (including struct return tests)
- [ ] Minimum 5 error handling tests pass
- [ ] Code follows existing patterns and structure
- [ ] No regressions in existing tests

## Future Enhancements

- Struct initialization with named members
- Default member initialization
- Struct arrays
- Structs in uniform blocks (std140 layout)

## Notes

- Start with simple structs (scalars only), then add vectors/matrices
- Memory layout follows std140 rules (can be simplified for local structs)
- Consider struct value vs struct pointer representation
- May need to handle large structs differently (pass by reference)
- JIT calling conventions: Structs are flattened into multiple return values (one per field, recursively), following the same pattern as matrices. This allows test functions to return structs and have their values verified.

