# Phase 4: Expression Codegen Splitting

## Current State

`codegen/expr.rs` is 1339 lines with all expression codegen logic:

- Literals (int, float, bool)
- Variables
- Binary/unary operators
- Function calls (built-in, user, constructors)
- Vector operations (200+ lines)
- Matrix operations (300+ lines)
- Component access/swizzling
- Type coercion

## Target Architecture

### Proposed Structure

```
codegen/
├── expr/
│   ├── mod.rs              - Public API, dispatcher
│   ├── literal.rs          - Literal translation
│   ├── variable.rs         - Variable access
│   ├── binary.rs           - Binary operator codegen
│   ├── unary.rs            - Unary operator codegen
│   ├── function.rs         - Function call codegen
│   ├── constructor.rs      - Type constructor codegen
│   ├── vector.rs           - Vector operation codegen
│   ├── matrix.rs           - Matrix operation codegen
│   ├── component.rs        - Component access/swizzling
│   └── coercion.rs         - Type coercion utilities
├── expr.rs                 - (remove, replaced by expr/ module)
```

## Refactoring Plan

### Step 1: Create Module Structure

**File**: `codegen/expr/mod.rs`

```rust
pub mod literal;
pub mod variable;
pub mod binary;
pub mod unary;
pub mod function;
pub mod constructor;
pub mod vector;
pub mod matrix;
pub mod component;
pub mod coercion;

use crate::codegen::context::CodegenContext;
use glsl::syntax::Expr;

impl<'a> CodegenContext<'a> {
    /// Main entry point for expression translation
    pub fn translate_expr_typed(
        &mut self,
        expr: &Expr,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        match expr {
            Expr::IntConst(..) | Expr::FloatConst(..) | Expr::BoolConst(..) => {
                literal::translate_literal(self, expr)
            }
            Expr::Variable(..) => {
                variable::translate_variable(self, expr)
            }
            Expr::Binary(..) => {
                binary::translate_binary(self, expr)
            }
            Expr::Unary(..) => {
                unary::translate_unary(self, expr)
            }
            Expr::FunCall(..) => {
                function::translate_function_call(self, expr)
            }
            Expr::Dot(..) => {
                component::translate_component_access(self, expr)
            }
            Expr::Assignment(..) => {
                // Assignment is handled in stmt.rs, but expression result
                // needs to be computed here
                self.translate_assignment_expr(expr)
            }
            _ => Err(GlslError::new(ErrorCode::E0400, "unsupported expression")),
        }
    }
}
```

### Step 2: Extract Literal Translation

**File**: `codegen/expr/literal.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_literal(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, Type), GlslError> {
    match expr {
        Expr::IntConst(n, _) => {
            let val = ctx.builder.ins().iconst(types::I32, *n as i64);
            Ok((vec![val], Type::Int))
        }
        Expr::FloatConst(f, _) => {
            let val = ctx.builder.ins().f32const(*f);
            Ok((vec![val], Type::Float))
        }
        Expr::BoolConst(b, _) => {
            let val = ctx.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
            Ok((vec![val], Type::Bool))
        }
        _ => unreachable!("translate_literal called on non-literal"),
    }
}
```

**Lines to move**: ~15 lines

### Step 3: Extract Variable Translation

**File**: `codegen/expr/variable.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_variable(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, Type), GlslError> {
    let Expr::Variable(ident, _span) = expr else {
        unreachable!("translate_variable called on non-variable");
    };

    let span = extract_span_from_identifier(ident);
    let vars = ctx
        .lookup_variables(&ident.name)
        .ok_or_else(|| {
            let error = GlslError::undefined_variable(&ident.name)
                .with_location(source_span_to_location(&span));
            ctx.add_span_to_error(error, &span)
        })?
        .to_vec();

    let ty = ctx
        .lookup_variable_type(&ident.name)
        .ok_or_else(|| {
            let error = GlslError::new(ErrorCode::E0400, format!("variable type not found for `{}` during codegen", ident.name))
                .with_location(source_span_to_location(&span));
            ctx.add_span_to_error(error, &span)
        })?
        .clone();

    let vals: Vec<Value> = vars.iter()
        .map(|&v| ctx.builder.use_var(v))
        .collect();

    Ok((vals, ty))
}
```

**Lines to move**: ~30 lines

### Step 4: Extract Binary Operator Translation

**File**: `codegen/expr/binary.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_binary(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, Type), GlslError> {
    let Expr::Binary(op, lhs, rhs, span) = expr else {
        unreachable!("translate_binary called on non-binary expr");
    };

    let (lhs_vals, lhs_ty) = ctx.translate_expr_typed(lhs)?;
    let (rhs_vals, rhs_ty) = ctx.translate_expr_typed(rhs)?;

    // Delegate to matrix/vector/scalar handlers
    if lhs_ty.is_matrix() || rhs_ty.is_matrix() {
        matrix::translate_matrix_binary(ctx, op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, span.clone())
    } else if lhs_ty.is_vector() || rhs_ty.is_vector() {
        vector::translate_vector_binary(ctx, op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, Some(span.clone()))
    } else {
        translate_scalar_binary(ctx, op, lhs_vals[0], &lhs_ty, rhs_vals[0], &rhs_ty, span.clone())
    }
}

fn translate_scalar_binary(
    ctx: &mut CodegenContext,
    op: &BinaryOp,
    lhs_val: Value,
    lhs_ty: &Type,
    rhs_val: Value,
    rhs_ty: &Type,
    span: SourceSpan,
) -> Result<(Vec<Value>, Type), GlslError> {
    // Current scalar binary logic (~100 lines)
    // - Type promotion
    // - Operator translation (arithmetic, comparison, logical)
    // - Result type inference
}
```

**Lines to move**: ~120 lines (dispatcher + scalar logic)

### Step 5: Extract Unary Operator Translation

**File**: `codegen/expr/unary.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_unary(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, Type), GlslError> {
    let Expr::Unary(op, operand, span) = expr else {
        unreachable!("translate_unary called on non-unary expr");
    };

    let (operand_vals, operand_ty) = ctx.translate_expr_typed(operand)?;

    // Current unary logic (~50 lines)
    // - Minus, Not operators
    // - Type validation
    // - Vector/scalar handling
}
```

**Lines to move**: ~60 lines

### Step 6: Extract Function Call Translation

**File**: `codegen/expr/function.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_function_call(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, Type), GlslError> {
    let Expr::FunCall(func_ident, args, span) = expr else {
        unreachable!("translate_function_call called on non-call");
    };

    let func_name = match func_ident {
        FunIdentifier::Identifier(ident) => &ident.name,
        _ => return Err(GlslError::new(ErrorCode::E0400, "complex function identifiers not yet supported")),
    };

    // Check if it's a type constructor
    if is_vector_type_name(func_name) {
        return constructor::translate_vector_constructor(ctx, func_name, args, span.clone());
    }

    if is_matrix_type_name(func_name) {
        return constructor::translate_matrix_constructor(ctx, func_name, args);
    }

    // Check if it's a built-in function
    if is_builtin_function(func_name) {
        return builtins::translate_builtin_call(ctx, func_name, args, span.clone());
    }

    // User-defined function
    translate_user_function_call(ctx, func_name, args, span.clone())
}

fn translate_user_function_call(
    ctx: &mut CodegenContext,
    name: &str,
    args: &[Expr],
    span: SourceSpan,
) -> Result<(Vec<Value>, Type), GlslError> {
    // Current user function call logic (~100 lines)
    // - Argument translation
    // - Function lookup
    // - Call generation
    // - Return value handling
}
```

**Lines to move**: ~150 lines

### Step 7: Extract Constructor Translation

**File**: `codegen/expr/constructor.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_vector_constructor(
    ctx: &mut CodegenContext,
    type_name: &str,
    args: &[Expr],
    span: SourceSpan,
) -> Result<(Vec<Value>, Type), GlslError> {
    // Current vector constructor logic (~150 lines)
    // - Single scalar (broadcast)
    // - Single vector (conversion)
    // - Multiple args (concatenation)
}

pub fn translate_matrix_constructor(
    ctx: &mut CodegenContext,
    type_name: &str,
    args: &[Expr],
) -> Result<(Vec<Value>, Type), GlslError> {
    // Current matrix constructor logic (~200 lines)
    // - Single scalar (identity)
    // - Column vectors
    // - Mixed scalars (column-major)
}
```

**Lines to move**: ~350 lines

### Step 8: Extract Vector Operations

**File**: `codegen/expr/vector.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_vector_binary(
    ctx: &mut CodegenContext,
    op: &BinaryOp,
    lhs_vals: Vec<Value>,
    lhs_ty: &Type,
    rhs_vals: Vec<Value>,
    rhs_ty: &Type,
    span: Option<SourceSpan>,
) -> Result<(Vec<Value>, Type), GlslError> {
    // Current vector binary logic (~200 lines)
    // - Vector + Vector (component-wise)
    // - Vector + Scalar (broadcast)
    // - Comparison operators
    // - Component-wise operations
}
```

**Lines to move**: ~220 lines

### Step 9: Extract Matrix Operations

**File**: `codegen/expr/matrix.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_matrix_binary(
    ctx: &mut CodegenContext,
    op: &BinaryOp,
    lhs_vals: Vec<Value>,
    lhs_ty: &Type,
    rhs_vals: Vec<Value>,
    rhs_ty: &Type,
    span: SourceSpan,
) -> Result<(Vec<Value>, Type), GlslError> {
    // Current matrix binary logic (~300 lines)
    // - Matrix + Matrix (component-wise)
    // - Matrix × Scalar (component-wise)
    // - Matrix × Vector (linear algebra)
    // - Vector × Matrix (linear algebra)
    // - Matrix × Matrix (linear algebra)
    // - Matrix / Scalar (component-wise)
}
```

**Lines to move**: ~320 lines

### Step 10: Extract Component Access

**File**: `codegen/expr/component.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn translate_component_access(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, Type), GlslError> {
    let Expr::Dot(base_expr, field, dot_span) = expr else {
        unreachable!("translate_component_access called on non-dot expr");
    };

    // Current component access logic (~100 lines)
    // - Base expression translation
    // - Swizzle parsing
    // - Component extraction
    // - Result type determination
}
```

**Lines to move**: ~120 lines

### Step 11: Extract Type Coercion

**File**: `codegen/expr/coercion.rs`

```rust
use crate::codegen::context::CodegenContext;

pub fn coerce_to_type(
    ctx: &mut CodegenContext,
    value: Value,
    from_ty: &Type,
    to_ty: &Type,
) -> Result<Value, GlslError> {
    // Current coercion logic (~80 lines)
    // - Int → Float conversion
    // - Scalar → Vector broadcast
    // - Vector component type conversion
}
```

**Lines to move**: ~90 lines

## Module Dependencies

```
mod.rs
├── literal.rs
├── variable.rs
├── binary.rs
│   ├── vector.rs
│   ├── matrix.rs
│   └── coercion.rs
├── unary.rs
│   └── coercion.rs
├── function.rs
│   ├── constructor.rs
│   └── builtins.rs (existing)
├── constructor.rs
│   ├── vector.rs
│   ├── matrix.rs
│   └── coercion.rs
├── vector.rs
│   └── coercion.rs
├── matrix.rs
│   └── coercion.rs
└── component.rs
```

## Migration Strategy

1. **Create module structure** alongside existing file
2. **Move functions** one module at a time
3. **Update CodegenContext** to delegate to modules
4. **Update imports** gradually
5. **Add tests** for each module
6. **Remove old file** once migration complete

## Testing Strategy

- Unit tests for each module
- Integration tests for expression translation
- Ensure existing tests still pass
- Add module-specific tests for edge cases

## Benefits

1. **Clarity**: Each module handles one expression type
2. **Maintainability**: Easier to find and modify specific logic
3. **Testability**: Modules can be tested independently
4. **Reusability**: Modules can be composed for complex expressions
5. **Readability**: Smaller, focused files (~100-200 lines each vs 1339 lines)

## File Size Targets

- `mod.rs`: ~100 lines (dispatcher)
- `literal.rs`: ~20 lines
- `variable.rs`: ~35 lines
- `binary.rs`: ~120 lines
- `unary.rs`: ~60 lines
- `function.rs`: ~150 lines
- `constructor.rs`: ~350 lines
- `vector.rs`: ~220 lines
- `matrix.rs`: ~320 lines
- `component.rs`: ~120 lines
- `coercion.rs`: ~90 lines

**Total**: ~1585 lines (some increase due to module structure, but much better organized)
