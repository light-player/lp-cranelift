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
- [ ] Minimum 8 functionality tests pass
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

