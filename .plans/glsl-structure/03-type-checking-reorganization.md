# Phase 3: Type Checking Module Reorganization

## Current State

`semantic/type_check.rs` is 878 lines with mixed responsibilities:

- Type inference (expressions, binary/unary ops)
- Type conversion rules (promotion, implicit conversion)
- Constructor validation (vector, matrix)
- Swizzle parsing
- Matrix operation type inference (270+ lines)

## Target Architecture

### Proposed Structure

```
semantic/
├── type_check/
│   ├── mod.rs              - Public API, re-exports
│   ├── inference.rs        - Expression type inference
│   ├── conversion.rs       - Type promotion and conversion rules
│   ├── constructors.rs     - Constructor validation (vector/matrix)
│   ├── operators.rs        - Operator type inference (binary/unary)
│   ├── matrix.rs           - Matrix operation type inference
│   └── swizzle.rs          - Swizzle parsing and validation
├── type_check.rs           - (remove, replaced by type_check/ module)
```

## Refactoring Plan

### Step 1: Create Module Structure

**File**: `semantic/type_check/mod.rs`

```rust
pub mod inference;
pub mod conversion;
pub mod constructors;
pub mod operators;
pub mod matrix;
pub mod swizzle;

// Re-export public API
pub use inference::{infer_expr_type, infer_expr_type_with_registry};
pub use conversion::{promote_numeric, can_implicitly_convert};
pub use constructors::{
    check_vector_constructor, check_vector_constructor_with_span,
    check_matrix_constructor,
    is_vector_type_name, is_matrix_type_name,
};
pub use operators::{infer_binary_result_type, infer_unary_result_type};
pub use matrix::infer_matrix_binary_result_type;
pub use swizzle::parse_swizzle_length;
```

### Step 2: Extract Type Inference

**File**: `semantic/type_check/inference.rs`

Move expression type inference:

```rust
/// Infer the result type of an expression
pub fn infer_expr_type(
    expr: &Expr,
    symbols: &SymbolTable,
) -> Result<Type, GlslError> {
    infer_expr_type_with_registry(expr, symbols, None)
}

/// Infer the result type of an expression with optional function registry
pub fn infer_expr_type_with_registry(
    expr: &Expr,
    symbols: &SymbolTable,
    func_registry: Option<&FunctionRegistry>,
) -> Result<Type, GlslError> {
    // Current implementation (180 lines)
    // Delegates to operators for binary/unary
    // Delegates to constructors for type constructors
}
```

**Lines to move**: ~180 lines (current `infer_expr_type_with_registry`)

### Step 3: Extract Type Conversion

**File**: `semantic/type_check/conversion.rs`

Move conversion rules:

```rust
/// Promote numeric types (GLSL spec implicit conversion rules)
pub fn promote_numeric(lhs: &Type, rhs: &Type) -> Type {
    // Current implementation
}

/// Check if implicit conversion is allowed
pub fn can_implicitly_convert(from: &Type, to: &Type) -> bool {
    // Current implementation (25 lines)
}

/// Validate assignment types
pub fn check_assignment(lhs_ty: &Type, rhs_ty: &Type) -> Result<(), GlslError> {
    check_assignment_with_span(lhs_ty, rhs_ty, None)
}

/// Validate assignment types with optional span
pub fn check_assignment_with_span(
    lhs_ty: &Type,
    rhs_ty: &Type,
    span: Option<SourceSpan>,
) -> Result<(), GlslError> {
    // Current implementation
}
```

**Lines to move**: ~80 lines

### Step 4: Extract Constructor Validation

**File**: `semantic/type_check/constructors.rs`

Move constructor logic:

```rust
/// Check vector constructor arguments and infer result type
pub fn check_vector_constructor(
    type_name: &str,
    args: &[Type],
) -> Result<Type, GlslError> {
    check_vector_constructor_with_span(type_name, args, None)
}

/// Check vector constructor with optional span
pub fn check_vector_constructor_with_span(
    type_name: &str,
    args: &[Type],
    span: Option<SourceSpan>,
) -> Result<Type, GlslError> {
    // Current implementation (~90 lines)
}

/// Check matrix constructor arguments and infer result type
pub fn check_matrix_constructor(
    type_name: &str,
    args: &[Type],
) -> Result<Type, GlslError> {
    // Current implementation (~80 lines)
}

/// Check if a name is a vector type constructor
pub fn is_vector_type_name(name: &str) -> bool {
    // Current implementation
}

/// Check if a name is a matrix type constructor
pub fn is_matrix_type_name(name: &str) -> bool {
    // Current implementation
}

fn parse_vector_type_name(name: &str) -> Result<Type, GlslError> {
    // Helper
}

fn parse_matrix_type_name(name: &str) -> Result<Type, GlslError> {
    // Helper
}

fn count_total_components(args: &[Type]) -> Result<usize, GlslError> {
    // Helper
}
```

**Lines to move**: ~200 lines

### Step 5: Extract Operator Type Inference

**File**: `semantic/type_check/operators.rs`

Move operator logic:

```rust
/// Infer result type of binary operation (with implicit conversion)
pub fn infer_binary_result_type(
    op: &BinaryOp,
    lhs_ty: &Type,
    rhs_ty: &Type,
    span: SourceSpan,
) -> Result<Type, GlslError> {
    // Handle matrix operations by delegating to matrix module
    if lhs_ty.is_matrix() || rhs_ty.is_matrix() {
        return matrix::infer_matrix_binary_result_type(op, lhs_ty, rhs_ty, span);
    }

    // Current scalar/vector logic (~100 lines)
}

/// Infer result type of unary operation
pub fn infer_unary_result_type(
    op: &UnaryOp,
    operand_ty: &Type,
    span: SourceSpan,
) -> Result<Type, GlslError> {
    // Current implementation (~40 lines)
}

/// Validate condition expression type (must be bool)
pub fn check_condition(cond_ty: &Type) -> Result<(), GlslError> {
    // Current implementation
}
```

**Lines to move**: ~150 lines

### Step 6: Extract Matrix Operations

**File**: `semantic/type_check/matrix.rs`

Move matrix-specific logic:

```rust
/// Infer result type of matrix binary operation
/// Implements GLSL spec: operators.adoc:1019-1098
pub fn infer_matrix_binary_result_type(
    op: &BinaryOp,
    lhs_ty: &Type,
    rhs_ty: &Type,
    span: SourceSpan,
) -> Result<Type, GlslError> {
    // Current implementation (~165 lines)
    // Handles:
    // - Matrix + Matrix (component-wise)
    // - Matrix × Scalar (component-wise)
    // - Matrix × Vector (linear algebra)
    // - Vector × Matrix (linear algebra)
    // - Matrix × Matrix (linear algebra)
    // - Matrix / Scalar (component-wise)
}
```

**Lines to move**: ~170 lines

**Benefits**:

- Matrix operations are complex and deserve isolation
- Easier to understand matrix-specific rules
- Can be extended for matrix-specific optimizations

### Step 7: Extract Swizzle Parsing

**File**: `semantic/type_check/swizzle.rs`

Move swizzle logic:

```rust
/// Component naming sets for vector swizzles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamingSet {
    XYZW,  // Position/generic: x, y, z, w
    RGBA,  // Color: r, g, b, a
    STPQ,  // Texture coordinates: s, t, p, q
}

/// Parse swizzle string and return the number of components
/// Validates that the swizzle is valid for the given vector size
pub fn parse_swizzle_length(
    swizzle: &str,
    max_components: usize,
) -> Result<usize, GlslError> {
    // Current implementation (~60 lines)
    // Validates:
    // - Non-empty
    // - Max 4 components
    // - Consistent naming set (xyzw/rgba/stpq)
    // - Components within bounds
}
```

**Lines to move**: ~60 lines

**Benefits**:

- Swizzle parsing is self-contained
- Can be reused by codegen
- Easier to extend with new swizzle patterns

## Module Dependencies

```
inference.rs
├── operators.rs (for binary/unary ops)
├── constructors.rs (for type constructors)
├── matrix.rs (for matrix operations)
└── swizzle.rs (for component access)

operators.rs
├── conversion.rs (for type promotion)
└── matrix.rs (for matrix ops)

constructors.rs
└── conversion.rs (for implicit conversion)

matrix.rs
└── conversion.rs (for type promotion)
```

## Migration Strategy

1. **Create module structure** alongside existing file
2. **Move functions** one module at a time
3. **Update imports** gradually
4. **Add tests** for each module
5. **Remove old file** once migration complete

## Testing Strategy

- Unit tests for each module
- Ensure existing tests still pass
- Add module-specific tests for edge cases
- Test module interactions

## Benefits

1. **Clarity**: Each module has single responsibility
2. **Maintainability**: Easier to find and modify specific logic
3. **Testability**: Modules can be tested independently
4. **Reusability**: Modules can be used by codegen
5. **Readability**: Smaller, focused files (~150-200 lines each vs 878 lines)

## File Size Targets

- `inference.rs`: ~180 lines
- `conversion.rs`: ~80 lines
- `constructors.rs`: ~200 lines
- `operators.rs`: ~150 lines
- `matrix.rs`: ~170 lines
- `swizzle.rs`: ~60 lines
- `mod.rs`: ~30 lines

**Total**: ~870 lines (similar to original, but better organized)
