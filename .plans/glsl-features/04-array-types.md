# Array Types Implementation

## Overview

Implement fixed-size arrays with indexing, initialization, and assignment. Arrays enable collections of data essential for many shader algorithms.

**Spec Reference:** `variables.adoc` lines 916-1012, `operators.adoc` lines 327-371  
**Priority:** Medium  
**Estimated Effort:** 3-4 hours

## Current State

- ✅ `Type::Array(Box<Type>, usize)` exists in type system
- ❌ No semantic analysis for array declarations
- ❌ No codegen for array construction
- ❌ No codegen for array indexing
- ❌ Matrix indexing uses `Expr::Bracket` but only for matrices

## Requirements

### Array Declaration

```glsl
float values[4];                    // Uninitialized
float values[3] = float[3](1.0, 2.0, 3.0);  // Initialized
vec3 positions[2];                  // Array of vectors
```

**Requirements:**
- Array size must be compile-time constant
- Size must be > 0
- Arrays can be of any type (scalars, vectors, matrices, structs)
- Multi-dimensional arrays: `float matrix[3][3];` (arrays of arrays)

### Array Initialization

```glsl
float values[3] = float[3](1.0, 2.0, 3.0);
int data[5] = int[5](10, 20, 30, 40, 50);
```

**Requirements:**
- Constructor syntax: `Type[size](arg1, arg2, ...)`
- All elements must be provided
- Type of each argument must match element type (with implicit conversion)

### Array Indexing

```glsl
float x = values[1];
values[2] = 5.0;
vec3 pos = positions[0];
```

**Requirements:**
- Index must be integral type (int)
- Index can be compile-time or runtime constant
- Bounds checking (optional, for safety)
- Multi-dimensional: `matrix[i][j]`

### Array Assignment

Arrays cannot be assigned as a whole (per GLSL spec), only individual elements.

## Implementation Strategy

### 1. Type System (`semantic/types.rs`)

**Current state:** `Type::Array(Box<Type>, usize)` exists.

**Add helper methods:**

```rust
impl Type {
    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _))
    }
    
    pub fn array_element_type(&self) -> Option<&Type> {
        match self {
            Type::Array(elem_ty, _) => Some(elem_ty),
            _ => None,
        }
    }
    
    pub fn array_size(&self) -> Option<usize> {
        match self {
            Type::Array(_, size) => Some(*size),
            _ => None,
        }
    }
    
    pub fn array_size_bytes(&self) -> Option<usize> {
        match self {
            Type::Array(elem_ty, size) => {
                Some(elem_ty.size() * size)
            }
            _ => None,
        }
    }
}
```

### 2. Semantic Analysis (`semantic/passes/`)

**Extend type resolver:**

```rust
// In semantic/type_resolver.rs
pub fn resolve_array_type(
    base_type: Type,
    array_spec: &glsl::syntax::ArraySpecifier,
) -> Result<Type, GlslError> {
    let mut result_ty = base_type;
    
    // Process dimensions from innermost to outermost
    for dim in array_spec.dimensions.0.iter().rev() {
        match dim {
            glsl::syntax::ArraySpecifierDimension::ExplicitlySized(size_expr) => {
                // Evaluate compile-time constant
                let size = evaluate_constant_int(size_expr)?;
                if size <= 0 {
                    return Err(GlslError::new(ErrorCode::E0400,
                        format!("array size must be > 0, got {}", size)));
                }
                result_ty = Type::Array(Box::new(result_ty), size as usize);
            }
            glsl::syntax::ArraySpecifierDimension::Unsized => {
                return Err(GlslError::new(ErrorCode::E0400,
                    "unsized arrays not yet supported"));
            }
        }
    }
    
    Ok(result_ty)
}
```

**Array constructor checking:**

```rust
// In semantic/type_check/constructors.rs
pub fn check_array_constructor(
    elem_type: &Type,
    size: usize,
    args: &[&Expr],
    symbols: &SymbolTable,
) -> Result<Type, GlslError> {
    if args.len() != size {
        return Err(GlslError::new(ErrorCode::E0104,
            format!("array constructor expects {} elements, got {}", size, args.len())));
    }
    
    // Check each argument type matches element type
    for (i, arg) in args.iter().enumerate() {
        let arg_ty = infer_expr_type_with_registry(arg, symbols, None)?;
        if !can_implicitly_convert(&arg_ty, elem_type) {
            return Err(GlslError::new(ErrorCode::E0106,
                format!("array constructor element {}: expected {:?}, got {:?}", i, elem_type, arg_ty)));
        }
    }
    
    Ok(Type::Array(Box::new(elem_type.clone()), size))
}
```

### 3. Type Checking (`semantic/type_check/`)

**Array indexing checking:**

```rust
// In semantic/type_check/inference.rs
fn infer_array_indexing(
    array_expr: &Expr,
    index_expr: &Expr,
    symbols: &SymbolTable,
) -> Result<Type, GlslError> {
    let array_ty = infer_expr_type_with_registry(array_expr, symbols, None)?;
    
    let elem_ty = array_ty.array_element_type()
        .ok_or_else(|| GlslError::new(ErrorCode::E0106,
            format!("indexing requires array type, got {:?}", array_ty)))?;
    
    let index_ty = infer_expr_type_with_registry(index_expr, symbols, None)?;
    if !index_ty.is_integral() {
        return Err(GlslError::new(ErrorCode::E0106,
            format!("array index must be integral type, got {:?}", index_ty)));
    }
    
    // Optional: Check bounds for compile-time constant indices
    if let Some(const_index) = evaluate_constant_int(index_expr) {
        let array_size = array_ty.array_size().unwrap();
        if const_index < 0 || const_index >= array_size as i32 {
            return Err(GlslError::new(ErrorCode::E0400,
                format!("array index {} out of bounds [0, {})", const_index, array_size)));
        }
    }
    
    Ok(elem_ty.clone())
}
```

### 4. Code Generation (`codegen/`)

**Array storage:**

Arrays are stored on the stack as consecutive memory.

```rust
// In codegen/context.rs
pub struct ArrayValue {
    pub elem_type: Type,
    pub size: usize,
    pub base_addr: Value,  // Pointer to array on stack
}
```

**Array construction:**

```rust
// In codegen/expr/constructor.rs
pub fn translate_array_constructor(
    ctx: &mut CodegenContext,
    elem_type: &GlslType,
    size: usize,
    args: &[Expr],
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let elem_size = elem_type.size();
    let array_size = elem_size * size;
    
    // Allocate stack space for array
    let array_ptr = ctx.allocate_stack_slot(array_size);
    
    // Evaluate each argument and store at element offset
    for (i, arg_expr) in args.iter().enumerate() {
        let (arg_vals, arg_ty) = ctx.translate_expr_typed(arg_expr)?;
        
        // Coerce to element type if needed
        let coerced_vals = coerce_to_type(ctx, arg_vals, &arg_ty, elem_type)?;
        
        // Store at element offset
        let offset = i * elem_size;
        store_at_offset(ctx, array_ptr, offset, coerced_vals, elem_type)?;
    }
    
    // Return pointer to array
    Ok((vec![array_ptr], GlslType::Array(Box::new(elem_type.clone()), size)))
}
```

**Array indexing:**

```rust
// In codegen/expr/component.rs (extend existing)
pub fn translate_array_indexing(
    ctx: &mut CodegenContext,
    array_expr: &Expr,
    index_expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let (array_vals, array_ty) = ctx.translate_expr_typed(array_expr)?;
    
    let elem_ty = array_ty.array_element_type()
        .ok_or_else(|| GlslError::new(ErrorCode::E0106, "indexing requires array"))?;
    
    let (index_vals, index_ty) = ctx.translate_expr_typed(index_expr)?;
    let index_val = index_vals[0];
    
    // Convert index to i32 if needed
    let index_i32 = coerce_to_type(ctx, vec![index_val], &index_ty, &GlslType::Int)?[0];
    
    // Calculate element offset
    let elem_size = elem_ty.size();
    let elem_size_val = ctx.builder.ins().iconst(types::I32, elem_size as i64);
    let offset_bytes = ctx.builder.ins().imul(index_i32, elem_size_val);
    
    // Load element at offset
    let array_ptr = array_vals[0];
    let elem_ptr = ctx.builder.ins().iadd(array_ptr, offset_bytes);
    let elem_vals = load_at_address(ctx, elem_ptr, elem_ty)?;
    
    Ok((elem_vals, elem_ty.clone()))
}
```

**Array element assignment:**

```rust
// In codegen/stmt.rs (extend assignment handling)
fn translate_array_element_assignment(
    ctx: &mut CodegenContext,
    array_expr: &Expr,
    index_expr: &Expr,
    rhs: &Expr,
) -> Result<(), GlslError> {
    let (array_vals, array_ty) = ctx.translate_expr_typed(array_expr)?;
    
    let elem_ty = array_ty.array_element_type()
        .ok_or_else(|| GlslError::new(ErrorCode::E0106, "indexing requires array"))?;
    
    let (index_vals, index_ty) = ctx.translate_expr_typed(index_expr)?;
    let index_val = index_vals[0];
    let index_i32 = coerce_to_type(ctx, vec![index_val], &index_ty, &GlslType::Int)?[0];
    
    let (rhs_vals, rhs_ty) = ctx.translate_expr_typed(rhs)?;
    let coerced_vals = coerce_to_type(ctx, rhs_vals, &rhs_ty, elem_ty)?;
    
    // Calculate element offset and store
    let elem_size = elem_ty.size();
    let elem_size_val = ctx.builder.ins().iconst(types::I32, elem_size as i64);
    let offset_bytes = ctx.builder.ins().imul(index_i32, elem_size_val);
    
    let array_ptr = array_vals[0];
    let elem_ptr = ctx.builder.ins().iadd(array_ptr, offset_bytes);
    store_at_address(ctx, elem_ptr, coerced_vals, elem_ty)?;
    
    Ok(())
}
```

## Testing Strategy

### Functionality Tests

**Location:** `crates/lp-glsl-filetests/filetests/arrays/`

**Basic Array:**
```glsl
// Test: array_declaration.glsl
// Spec: variables.adoc:916-1012
float main() {
    float values[3] = float[3](1.0, 2.0, 3.0);
    return values[1];
}
// run: == 2.0
```

**Array Indexing:**
```glsl
// Test: array_indexing.glsl
// Spec: variables.adoc:916-1012
int main() {
    int data[5] = int[5](10, 20, 30, 40, 50);
    return data[3];
}
// run: == 40
```

**Array Assignment:**
```glsl
// Test: array_assign.glsl
// Spec: variables.adoc:916-1012
float main() {
    float vals[2];
    vals[0] = 5.0;
    vals[1] = 10.0;
    return vals[0] + vals[1];
}
// run: == 15.0
```

**Array of Vectors:**
```glsl
// Test: array_vec3.glsl
// Spec: variables.adoc:916-1012
vec3 main() {
    vec3 positions[2] = vec3[2](vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
    return positions[1];
}
// run: == vec3(4.0, 5.0, 6.0)
```

**Multi-dimensional:**
```glsl
// Test: array_2d.glsl
// Spec: variables.adoc:916-1012
float main() {
    float matrix[2][3];
    matrix[0][0] = 1.0;
    matrix[0][1] = 2.0;
    matrix[1][0] = 3.0;
    return matrix[0][0] + matrix[1][0];
}
// run: == 4.0
```

### Error Handling Tests

**Location:** `crates/lp-glsl-filetests/filetests/type_errors/`

```glsl
// Test: array_constructor_wrong_count.glsl
float main() {
    float arr[3] = float[3](1.0, 2.0);  // ERROR: missing element
}
// EXPECT_ERROR: array constructor expects 3 elements, got 2

// Test: array_constructor_wrong_type.glsl
float main() {
    float arr[2] = float[2](1.0, true);  // ERROR: wrong type
}
// EXPECT_ERROR: array constructor element 1: expected Float, got Bool

// Test: array_index_out_of_bounds.glsl
float main() {
    float arr[3] = float[3](1.0, 2.0, 3.0);
    return arr[5];  // ERROR: index out of bounds
}
// EXPECT_ERROR: array index 5 out of bounds [0, 3)

// Test: array_index_wrong_type.glsl
float main() {
    float arr[3] = float[3](1.0, 2.0, 3.0);
    return arr[true];  // ERROR: index must be int
}
// EXPECT_ERROR: array index must be integral type, got Bool

// Test: array_zero_size.glsl
float arr[0];  // ERROR: size must be > 0
// EXPECT_ERROR: array size must be > 0, got 0
```

## Success Criteria

- [ ] Fixed-size array declarations work
- [ ] Array initialization with constructor
- [ ] Array indexing generates correct offsets
- [ ] Array element assignment works
- [ ] Arrays of scalars, vectors, structs
- [ ] Multi-dimensional arrays (arrays of arrays)
- [ ] Minimum 8 functionality tests pass
- [ ] Minimum 5 error handling tests pass
- [ ] Code follows existing patterns and structure
- [ ] No regressions in existing tests

## Future Enhancements

- Runtime bounds checking (optional)
- Arrays in structs
- Array length method
- Unsized arrays (for function parameters)

## Notes

- Start with 1D arrays, then add multi-dimensional
- Index can be runtime value (not just compile-time constant)
- Consider bounds checking as optional feature flag
- Array assignment (whole array) is not allowed per spec

