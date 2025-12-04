# GLSL Compiler - Phase 3: Float Type & Type System

## Overview

Add float type support with **proper type checking**, **type inference**, and **type coercion** according to the GLSL specification.

**Spec Reference:** `/Users/yona/dev/photomancer/glsl-spec/chapters/`

- `variables.adoc` lines 1182-1229: Implicit Conversions
- `operators.adoc` lines 775-855: Arithmetic operator type rules

## Current State (Phases 1-2)

**What Works:**

- ✅ int/bool types with arithmetic and comparisons
- ✅ Control flow (if/else, loops, break/continue)
- ✅ Variable declarations and assignments

**What's Missing:**

- ❌ NO type inference (don't track expression result types)
- ❌ NO type validation (can add int + bool without error!)
- ❌ NO type coercion (can't mix int and float)
- ❌ float type not implemented
- ❌ Limited error messages

**Example of Current Problem:**

```glsl
int main() {
    int x = 5;
    bool y = true;
    return x + y;  // Currently COMPILES (shouldn't!)
}
```

## GLSL Spec Type Rules (Phase 3 Subset)

### Implicit Conversions (Spec: variables.adoc:1182-1229)

According to GLSL spec, these conversions are allowed:

| From Type | Can Convert To |
| --------- | -------------- |
| **int**   | **float**      |

**NOT allowed:** bool → int, bool → float, float → int (must use constructor)

### Arithmetic Operators (Spec: operators.adoc:775-855)

**Rules:**

1. Operands must have matching fundamental types (after implicit conversion)
2. Result type = common type of operands (after promotion)
3. Valid cases:
   - Both scalars → scalar result
   - Scalar + vector → vector result (scalar applied component-wise)
   - Both vectors (same size) → vector result (component-wise)

**Phase 3 Scope:** Scalars only (int, float, bool)

### Comparison Operators (Spec: operators.adoc:876-884)

**Rules:**

1. Operands must have matching types (after implicit conversion)
2. Result is always **bool**
3. For component-wise vector comparison, use built-in functions (later phase)

### Assignment (Spec: operators.adoc:694-713)

**Rules:**

1. RHS must match LHS type OR have implicit conversion to LHS type
2. Example: `float x = 5;` is valid (int 5 → float conversion)
3. Example: `int x = 5.5;` is ERROR (no implicit float → int)

### Conditions (Spec: statements.adoc)

**Rule:** Condition expressions in if/while/for must be **bool** type

## Architecture

### Type Inference Infrastructure

**File: `crates/lp-glsl/src/semantic/type_check.rs`** (NEW)

```rust
use crate::semantic::types::Type;
use crate::semantic::scope::SymbolTable;
use glsl::syntax::{Expr, BinaryOp, UnaryOp};

/// Infer the result type of an expression
pub fn infer_expr_type(
    expr: &Expr,
    symbols: &SymbolTable,
) -> Result<Type, String> {
    match expr {
        Expr::IntConst(_) => Ok(Type::Int),
        Expr::FloatConst(_) => Ok(Type::Float),
        Expr::BoolConst(_) => Ok(Type::Bool),
        Expr::DoubleConst(_) => Ok(Type::Double),  // Future

        Expr::Variable(ident) => {
            let var = symbols.lookup_variable(&ident.0)
                .ok_or_else(|| format!("Undefined variable: {}", ident.0))?;
            Ok(var.ty.clone())
        }

        Expr::Binary(op, lhs, rhs) => {
            let lhs_ty = infer_expr_type(lhs, symbols)?;
            let rhs_ty = infer_expr_type(rhs, symbols)?;
            infer_binary_result_type(op, &lhs_ty, &rhs_ty)
        }

        Expr::Unary(op, expr) => {
            let expr_ty = infer_expr_type(expr, symbols)?;
            infer_unary_result_type(op, &expr_ty)
        }

        Expr::Assignment(lhs, _op, rhs) => {
            // Assignment result has same type as LHS
            infer_expr_type(lhs, symbols)
        }

        _ => Err(format!("Cannot infer type for: {:?}", expr)),
    }
}

/// Infer result type of binary operation (with implicit conversion)
pub fn infer_binary_result_type(
    op: &BinaryOp,
    lhs_ty: &Type,
    rhs_ty: &Type,
) -> Result<Type, String> {
    use BinaryOp::*;

    match op {
        // Arithmetic operators: operands must be numeric
        Add | Sub | Mult | Div => {
            if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
                return Err(format!(
                    "Arithmetic operator {:?} requires numeric operands, got {:?} and {:?}",
                    op, lhs_ty, rhs_ty
                ));
            }
            // Result type is the promoted type
            Ok(promote_numeric(lhs_ty, rhs_ty))
        }

        // Comparison operators: operands must be compatible, result is bool
        Equal | NonEqual | LT | GT | LTE | GTE => {
            if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
                return Err(format!(
                    "Comparison operator requires numeric operands, got {:?} and {:?}",
                    lhs_ty, rhs_ty
                ));
            }
            Ok(Type::Bool)
        }

        // Logical operators: must be bool
        And | Or | Xor => {
            if lhs_ty != &Type::Bool || rhs_ty != &Type::Bool {
                return Err(format!(
                    "Logical operator {:?} requires bool operands, got {:?} and {:?}",
                    op, lhs_ty, rhs_ty
                ));
            }
            Ok(Type::Bool)
        }

        _ => Err(format!("Unsupported binary operator: {:?}", op)),
    }
}

/// Promote numeric types (GLSL spec implicit conversion rules)
pub fn promote_numeric(lhs: &Type, rhs: &Type) -> Type {
    match (lhs, rhs) {
        (Type::Int, Type::Int) => Type::Int,
        (Type::Float, Type::Float) => Type::Float,
        (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
        // int → float implicit conversion per GLSL spec
        _ => Type::Int, // Fallback (shouldn't reach here after validation)
    }
}

/// Check if implicit conversion is allowed (GLSL spec: variables.adoc:1182-1229)
pub fn can_implicitly_convert(from: &Type, to: &Type) -> bool {
    from == to || matches!((from, to), (Type::Int, Type::Float))
}

/// Validate assignment types
pub fn check_assignment(lhs_ty: &Type, rhs_ty: &Type) -> Result<(), String> {
    if !can_implicitly_convert(rhs_ty, lhs_ty) {
        return Err(format!(
            "Type mismatch: cannot assign {:?} to {:?}",
            rhs_ty, lhs_ty
        ));
    }
    Ok(())
}

/// Validate condition expression type (must be bool)
pub fn check_condition(cond_ty: &Type) -> Result<(), String> {
    if cond_ty != &Type::Bool {
        return Err(format!(
            "Condition must be bool type, got {:?}",
            cond_ty
        ));
    }
    Ok(())
}
```

### Update Type System

**File: `crates/lp-glsl/src/semantic/types.rs`**

```rust
impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self, Type::Bool | Type::Int | Type::Float)
    }

    pub fn to_cranelift_type(&self) -> cranelift_codegen::ir::Type {
        match self {
            Type::Bool => cranelift_codegen::ir::types::I8,
            Type::Int => cranelift_codegen::ir::types::I32,
            Type::Float => cranelift_codegen::ir::types::F32,
            Type::Void => panic!("Void type has no Cranelift representation"),
            _ => panic!("Type not yet supported"),
        }
    }
}
```

### Update Codegen with Type Tracking

**File: `crates/lp-glsl/src/codegen/expr.rs`**

Changes:

1. `translate_expr` returns `Result<(Value, Type), String>` instead of `Result<Value, String>`
2. Track types through all expression translation
3. Implement type coercion when needed

```rust
pub fn translate_expr(&mut self, expr: &Expr) -> Result<(Value, Type), String> {
    match expr {
        Expr::IntConst(n) => {
            let val = self.builder.ins().iconst(types::I32, *n as i64);
            Ok((val, Type::Int))
        }

        Expr::FloatConst(f) => {
            let val = self.builder.ins().f32const(*f);
            Ok((val, Type::Float))
        }

        Expr::Binary(op, lhs, rhs) => {
            let (lhs_val, lhs_ty) = self.translate_expr(lhs)?;
            let (rhs_val, rhs_ty) = self.translate_expr(rhs)?;

            // Infer result type and validate
            let result_ty = infer_binary_result_type(op, &lhs_ty, &rhs_ty)?;

            // Promote operands to common type
            let common_ty = promote_numeric(&lhs_ty, &rhs_ty);
            let lhs_val = self.coerce_to_type(lhs_val, &lhs_ty, &common_ty)?;
            let rhs_val = self.coerce_to_type(rhs_val, &rhs_ty, &common_ty)?;

            // Generate operation
            let result_val = self.translate_binary_op(op, lhs_val, rhs_val, &common_ty)?;
            Ok((result_val, result_ty))
        }

        // ...
    }
}

fn coerce_to_type(
    &mut self,
    val: Value,
    from_ty: &Type,
    to_ty: &Type,
) -> Result<Value, String> {
    if from_ty == to_ty {
        return Ok(val);
    }

    match (from_ty, to_ty) {
        (Type::Int, Type::Float) => {
            // int → float: fcvt_from_sint
            Ok(self.builder.ins().fcvt_from_sint(types::F32, val))
        }
        _ => Err(format!("Cannot convert {:?} to {:?}", from_ty, to_ty)),
    }
}

fn translate_binary_op(
    &mut self,
    op: &BinaryOp,
    lhs: Value,
    rhs: Value,
    operand_ty: &Type,
) -> Result<Value, String> {
    use BinaryOp::*;

    let val = match op {
        Add => match operand_ty {
            Type::Int => self.builder.ins().iadd(lhs, rhs),
            Type::Float => self.builder.ins().fadd(lhs, rhs),
            _ => return Err(format!("Add not supported for {:?}", operand_ty)),
        },
        Sub => match operand_ty {
            Type::Int => self.builder.ins().isub(lhs, rhs),
            Type::Float => self.builder.ins().fsub(lhs, rhs),
            _ => return Err(format!("Sub not supported for {:?}", operand_ty)),
        },
        Mult => match operand_ty {
            Type::Int => self.builder.ins().imul(lhs, rhs),
            Type::Float => self.builder.ins().fmul(lhs, rhs),
            _ => return Err(format!("Mult not supported for {:?}", operand_ty)),
        },
        Div => match operand_ty {
            Type::Int => self.builder.ins().sdiv(lhs, rhs),
            Type::Float => self.builder.ins().fdiv(lhs, rhs),
            _ => return Err(format!("Div not supported for {:?}", operand_ty)),
        },

        // Comparisons return bool
        Equal => match operand_ty {
            Type::Int => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
            Type::Float => self.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
            _ => return Err(format!("Equal not supported for {:?}", operand_ty)),
        },

        // ... similar for other comparisons

        _ => return Err(format!("Unsupported operator: {:?}", op)),
    };

    Ok(val)
}
```

## Test Cases - Valid Programs

### Test 1: Float Literal

```glsl
// test compile
// test run

float main() {
    return 3.14;
}

// CHECK: f32const
// CHECK: return
// run: == 3.14
```

### Test 2: Float Arithmetic

```glsl
// test compile
// test run

float main() {
    float a = 2.5;
    float b = 1.5;
    return a + b;
}

// CHECK: f32const
// CHECK: fadd
// run: == 4.0
```

### Test 3: Float Multiplication

```glsl
// test compile
// test run

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// CHECK: fmul
// run: == 7.0
```

### Test 4: Float Comparison

```glsl
// test compile
// test run

bool main() {
    float a = 2.5;
    float b = 1.5;
    return a > b;
}

// CHECK: fcmp gt
// run: == true
```

### Test 5: Int to Float Implicit Conversion

```glsl
// test compile
// test run

float main() {
    int x = 5;
    float y = 2.5;
    return x + y;  // x implicitly converted to float
}

// CHECK: iconst.i32 5
// CHECK: fcvt_from_sint
// CHECK: f32const 0x1.4p1
// CHECK: fadd
// run: == 7.5
```

### Test 6: Float Assignment from Int

```glsl
// test compile
// test run

float main() {
    float x = 10;  // int 10 → float conversion
    return x;
}

// CHECK: iconst.i32 10
// CHECK: fcvt_from_sint
// run: == 10.0
```

### Test 7: Mixed Arithmetic

```glsl
// test compile
// test run

float main() {
    int a = 3;
    float b = 2.0;
    float c = a * b;  // 3 → 3.0, then 3.0 * 2.0
    return c;
}

// CHECK: fcvt_from_sint
// CHECK: fmul
// run: == 6.0
```

### Test 8: Float in Control Flow

```glsl
// test compile
// test run

float main() {
    float sum = 0.0;
    for (int i = 0; i < 3; i = i + 1) {
        sum = sum + 1.5;
    }
    return sum;
}

// CHECK: f32const
// CHECK: fadd
// run: == 4.5
```

## Test Cases - Type Errors (Should Fail)

**Reference:** GLSL spec - these should all be compile-time errors

### Error 1: Bool + Int (Spec: operators.adoc:775)

```glsl
// test error

int main() {
    int x = 5;
    bool y = true;
    return x + y;  // ERROR: cannot add int and bool
}

// EXPECT_ERROR: Arithmetic operator .* requires numeric operands
```

### Error 2: Float → Int Assignment (Spec: variables.adoc:1182)

```glsl
// test error

int main() {
    int x = 3.14;  // ERROR: no implicit float → int
    return x;
}

// EXPECT_ERROR: cannot assign.*Float.*to.*Int
```

### Error 3: Int Condition (Spec: statements.adoc)

```glsl
// test error

int main() {
    int x = 5;
    if (x) {  // ERROR: condition must be bool
        return 1;
    }
    return 0;
}

// EXPECT_ERROR: Condition must be bool type
```

### Error 4: Bool Arithmetic (Spec: operators.adoc:775)

```glsl
// test error

bool main() {
    bool a = true;
    bool b = false;
    return a + b;  // ERROR: cannot add bools
}

// EXPECT_ERROR: Arithmetic operator .* requires numeric operands
```

### Error 5: Type Mismatch in Assignment

```glsl
// test error

int main() {
    int x;
    bool y = true;
    x = y;  // ERROR: cannot assign bool to int
    return x;
}

// EXPECT_ERROR: cannot assign.*Bool.*to.*Int
```

### Error 6: Comparison of Incompatible Types

```glsl
// test error

bool main() {
    int x = 5;
    bool y = true;
    return x == y;  // ERROR: cannot compare int and bool
}

// EXPECT_ERROR: Comparison.*requires.*matching types
```

## Type Error Testing Infrastructure

### New Test Type: `test error`

**File: `crates/lp-glsl-filetests/src/test_error.rs`** (NEW)

```rust
//! Test that compilation fails with expected error

use anyhow::Result;

pub fn run_test(full_source: &str, glsl_source: &str) -> Result<()> {
    // Extract expected error pattern
    let error_pattern = extract_error_pattern(full_source)?;

    // Compile and expect failure
    let mut compiler = lp_glsl::Compiler::new();
    match compiler.compile_int(glsl_source) {
        Ok(_) => {
            anyhow::bail!("Expected compilation to fail, but it succeeded");
        }
        Err(e) => {
            // Check that error matches expected pattern
            if !error_matches(&e, &error_pattern) {
                anyhow::bail!(
                    "Error mismatch:\nExpected pattern: {}\nActual error: {}",
                    error_pattern,
                    e
                );
            }
        }
    }

    Ok(())
}

fn extract_error_pattern(source: &str) -> Result<String> {
    for line in source.lines() {
        if let Some(comment) = line.trim().strip_prefix("//") {
            if let Some(pattern) = comment.trim().strip_prefix("EXPECT_ERROR:") {
                return Ok(pattern.trim().to_string());
            }
        }
    }
    anyhow::bail!("No EXPECT_ERROR directive found")
}

fn error_matches(error: &str, pattern: &str) -> bool {
    // Simple substring match (could use regex in future)
    error.contains(pattern) ||
    error.to_lowercase().contains(&pattern.to_lowercase())
}
```

### Update Filetest Runner

**File: `crates/lp-glsl-filetests/src/filetest.rs`**

```rust
pub fn run_filetest(path: &Path) -> Result<()> {
    let source = fs::read_to_string(path)?;

    let test_compile = source.contains("test compile");
    let test_run = source.contains("test run");
    let test_error = source.contains("test error");  // NEW

    let glsl_source = extract_glsl_source(&source);

    if test_error {
        crate::test_error::run_test(&source, &glsl_source)?;
    }

    if test_compile {
        crate::test_compile::run_test(&source, &glsl_source)?;
    }

    if test_run {
        crate::test_run::run_test(&source, &glsl_source)?;
    }

    Ok(())
}
```

## Implementation Order

1. **Update Type system** - Add is_numeric(), is_scalar(), float support
2. **Create type_check.rs** - Type inference and validation functions
3. **Update semantic analysis** - Build symbol table, validate declarations
4. **Update codegen context** - Track variable types
5. **Update expr.rs** - Return (Value, Type), validate types
6. **Implement float literals** - Parse and generate f32const
7. **Implement float arithmetic** - fadd, fsub, fmul, fdiv
8. **Implement type coercion** - fcvt_from_sint for int → float
9. **Implement float comparisons** - fcmp for <, >, ==, !=, <=, >=
10. **Validate binary ops** - Check operand types before codegen
11. **Validate unary ops** - Check operand types
12. **Validate assignments** - Check RHS can convert to LHS
13. **Validate conditions** - Check if/while/for conditions are bool
14. **Create float filetests** - 8 valid float tests
15. **Create error filetests** - 6 type error tests
16. **Implement test error** - Error testing infrastructure
17. **Run all tests** - Validate Phase 3 complete

## File Structure

```
crates/lp-glsl/
├── src/
│   └── semantic/
│       ├── types.rs          (UPDATE: add is_numeric, float support)
│       ├── type_check.rs     (NEW: type inference/validation)
│       └── mod.rs            (UPDATE: build symbol table)

crates/lp-glsl-filetests/
├── src/
│   ├── test_error.rs         (NEW: error testing)
│   └── filetest.rs           (UPDATE: add test error support)
├── filetests/
│   ├── float/                (NEW)
│   │   ├── float_literal.glsl
│   │   ├── float_arithmetic.glsl
│   │   ├── float_comparison.glsl
│   │   ├── int_to_float.glsl
│   │   ├── mixed_arithmetic.glsl
│   │   ├── float_assignment.glsl
│   │   ├── float_complex.glsl
│   │   └── float_in_loop.glsl
│   └── type_errors/          (NEW)
│       ├── bool_plus_int.glsl
│       ├── float_to_int_assign.glsl
│       ├── int_condition.glsl
│       ├── bool_arithmetic.glsl
│       ├── type_mismatch_assign.glsl
│       └── incompatible_comparison.glsl
```

## Success Criteria

### Type System

- [ ] Float type works (literals, arithmetic, comparisons)
- [ ] Type inference tracks all expression types
- [ ] Type validation catches mismatches
- [ ] Implicit int → float conversion works
- [ ] Explicit conversions blocked (float → int requires constructor)

### Correctness

- [ ] All float operations produce correct results
- [ ] Type coercion happens automatically when allowed
- [ ] Type errors are caught at compile time
- [ ] Error messages match GLSL spec semantics

### Testing

- [ ] 8 float filetests pass (compile + run)
- [ ] 6 type error filetests correctly reject invalid code
- [ ] All Phase 1-2 tests still pass
- [ ] Example programs with floats work

## GLSL Spec Compliance

All type rules implemented according to:

- **Implicit Conversions:** `/Users/yona/dev/photomancer/glsl-spec/chapters/variables.adoc:1182-1229`
- **Arithmetic Operators:** `/Users/yona/dev/photomancer/glsl-spec/chapters/operators.adoc:775-855`
- **Comparison Operators:** `/Users/yona/dev/photomancer/glsl-spec/chapters/operators.adoc:876-884`
- **Assignment Rules:** `/Users/yona/dev/photomancer/glsl-spec/chapters/operators.adoc:694-713`

Phase 3 implements the scalar type subset (int, float, bool) per GLSL 4.60 specification.

