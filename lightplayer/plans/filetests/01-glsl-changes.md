# Compiler Changes (lp-glsl crate)

**Location:** `lightplayer/crates/lp-glsl/src/`

## Overview

Add compiler support for parsing programs, inferring expression types, and parsing literal values. These changes enable the filetests infrastructure to work with GLSL programs.

## Required Changes

### A. Add `GlslValue::parse()` method

**File:** `lightplayer/crates/lp-glsl/src/backend/executable.rs`

**Add method:**

```rust
impl GlslValue {
    /// Parse a literal value string into GlslValue using GLSL parser
    /// Only supports literals: integers, floats, booleans
    /// Uses type checking to ensure valid literal syntax
    pub fn parse(literal_str: &str) -> Result<Self, GlslError> {
        // Parse as expression: "42", "3.14", "true", etc.
        // Type check to ensure it's a literal
        // Convert to GlslValue
    }
}
```

**Requirements:**

- Use GLSL parser to parse the literal string as an expression
- Type check to ensure it's a literal (IntConst, FloatConst, BoolConst)
- Convert to appropriate `GlslValue` variant
- Support: `"0"`, `"42"`, `"-1"` → `GlslValue::I32`
- Support: `"0.0"`, `"1.5"`, `"-3.14"` → `GlslValue::F32`
- Support: `"true"`, `"false"` → `GlslValue::Bool`
- Error if not a literal or unsupported type

**Unit tests:** `lightplayer/crates/lp-glsl/src/backend/executable.rs` or separate test file

**Test cases:**

- Valid integers: `"0"`, `"42"`, `"-1"`
- Valid floats: `"0.0"`, `"1.5"`, `"-3.14"`
- Valid booleans: `"true"`, `"false"`
- Invalid: `"not_a_literal"`, `"x + y"`, `"add(1, 2)"`

### B. Add program parsing with function registry extraction

**File:** `lightplayer/crates/lp-glsl/src/compiler/pipeline.rs` or new helper module

**Add function:**

```rust
/// Parse and type-check a GLSL program, returning function registry
pub fn parse_program_with_registry(source: &str) -> Result<FunctionRegistry, GlslError> {
    // Parse program
    // Run semantic analysis
    // Extract function registry
    // Return registry for use in expression type inference
}
```

**Requirements:**

- Parse GLSL source code
- Run semantic analysis (type checking)
- Extract `FunctionRegistry` from semantic result
- Return registry that can be used for expression type inference
- Handle parse errors, type errors gracefully

**Unit tests:** Test with simple programs containing functions

**Test cases:**

- Simple program with one function
- Program with multiple functions
- Program with no functions (empty registry)
- Invalid programs (parse errors, type errors)

### C. Add expression type inference in context

**File:** `lightplayer/crates/lp-glsl/src/semantic/type_check/inference.rs` (may already exist)

**Verify/Enhance:**

```rust
/// Infer type of expression within a program context
pub fn infer_expr_type_in_context(
    expr_str: &str,
    function_registry: &FunctionRegistry,
) -> Result<Type, GlslError> {
    // Parse expression string
    // Use infer_expr_type_with_registry() with the provided registry
    // Return inferred type
}
```

**Requirements:**

- Parse expression string (e.g., `"add_float(0.0, 0.0)"`)
- Use existing `infer_expr_type_with_registry()` with provided function registry
- Return `Type` (Int, Float, Bool, etc.)
- Handle parse errors, unknown functions, type errors

**Note:** May already exist or be straightforward wrapper around existing function

**Unit tests:** Test with various expressions referencing functions in registry

**Test cases:**

- Function call: `"add_float(0.0, 0.0)"` → `Float`
- Function call: `"add_int(1, 2)"` → `Int`
- Literal: `"42"` → `Int`
- Literal: `"3.14"` → `Float`
- Unknown function (should error)
- Invalid expression (should error)

### D. Add comparison operators for `GlslValue`

**File:** `lightplayer/crates/lp-glsl/src/backend/executable.rs`

**Add methods:**

```rust
impl GlslValue {
    /// Exact equality comparison (==)
    /// For integers and booleans: exact match required
    /// For floats: exact match required (use `approx_eq` for tolerance-based comparison)
    pub fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GlslValue::I32(a), GlslValue::I32(b)) => a == b,
            (GlslValue::F32(a), GlslValue::F32(b)) => a == b,  // Exact equality
            (GlslValue::Bool(a), GlslValue::Bool(b)) => a == b,
            (GlslValue::Vec2(a), GlslValue::Vec2(b)) => a == b,
            (GlslValue::Vec3(a), GlslValue::Vec3(b)) => a == b,
            (GlslValue::Vec4(a), GlslValue::Vec4(b)) => a == b,
            (GlslValue::Mat2x2(a), GlslValue::Mat2x2(b)) => a == b,
            (GlslValue::Mat3x3(a), GlslValue::Mat3x3(b)) => a == b,
            (GlslValue::Mat4x4(a), GlslValue::Mat4x4(b)) => a == b,
            _ => false,  // Type mismatch
        }
    }

    /// Approximate equality comparison (~=) with tolerance
    /// For floats: checks if values are within tolerance
    /// For integers and booleans: falls back to exact equality
    /// For vectors/matrices: checks each component within tolerance
    pub fn approx_eq(&self, other: &Self, tolerance: f32) -> bool {
        match (self, other) {
            (GlslValue::I32(a), GlslValue::I32(b)) => a == b,  // Exact for ints
            (GlslValue::F32(a), GlslValue::F32(b)) => (a - b).abs() <= tolerance,
            (GlslValue::Bool(a), GlslValue::Bool(b)) => a == b,  // Exact for bools
            (GlslValue::Vec2(a), GlslValue::Vec2(b)) => {
                a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() <= tolerance)
            }
            (GlslValue::Vec3(a), GlslValue::Vec3(b)) => {
                a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() <= tolerance)
            }
            (GlslValue::Vec4(a), GlslValue::Vec4(b)) => {
                a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() <= tolerance)
            }
            (GlslValue::Mat2x2(a), GlslValue::Mat2x2(b)) => {
                a.iter().flatten().zip(b.iter().flatten())
                    .all(|(x, y)| (x - y).abs() <= tolerance)
            }
            (GlslValue::Mat3x3(a), GlslValue::Mat3x3(b)) => {
                a.iter().flatten().zip(b.iter().flatten())
                    .all(|(x, y)| (x - y).abs() <= tolerance)
            }
            (GlslValue::Mat4x4(a), GlslValue::Mat4x4(b)) => {
                a.iter().flatten().zip(b.iter().flatten())
                    .all(|(x, y)| (x - y).abs() <= tolerance)
            }
            _ => false,  // Type mismatch
        }
    }

    /// Default tolerance for float comparisons (1e-4)
    pub const DEFAULT_TOLERANCE: f32 = 1e-4;

    /// Approximate equality with default tolerance
    pub fn approx_eq_default(&self, other: &Self) -> bool {
        self.approx_eq(other, Self::DEFAULT_TOLERANCE)
    }
}
```

**Requirements:**

- `eq()`: Exact equality for all types

  - Integers: exact match
  - Floats: exact match (no tolerance)
  - Booleans: exact match
  - Vectors/Matrices: exact match for all components
  - Type mismatch returns `false`

- `approx_eq()`: Approximate equality with tolerance

  - Integers: falls back to exact equality (tolerance ignored)
  - Floats: checks `|a - b| <= tolerance`
  - Booleans: falls back to exact equality (tolerance ignored)
  - Vectors/Matrices: checks each component within tolerance
  - Type mismatch returns `false`

- `approx_eq_default()`: Convenience method using default tolerance (1e-4)

**Unit tests:** `lightplayer/crates/lp-glsl/src/backend/executable.rs` or separate test file

**Test cases:**

**Exact equality (`eq()`):**

- Integers: `I32(42).eq(&I32(42))` → `true`, `I32(42).eq(&I32(43))` → `false`
- Floats: `F32(1.0).eq(&F32(1.0))` → `true`, `F32(1.0).eq(&F32(1.0001))` → `false`
- Booleans: `Bool(true).eq(&Bool(true))` → `true`, `Bool(true).eq(&Bool(false))` → `false`
- Type mismatch: `I32(42).eq(&F32(42.0))` → `false`
- Vectors: `Vec2([1.0, 2.0]).eq(&Vec2([1.0, 2.0]))` → `true`
- Matrices: `Mat2x2([[1.0, 2.0], [3.0, 4.0]]).eq(&Mat2x2([[1.0, 2.0], [3.0, 4.0]]))` → `true`

**Approximate equality (`approx_eq()`):**

- Integers: `I32(42).approx_eq(&I32(42), 0.1)` → `true` (exact match required)
- Floats within tolerance: `F32(1.0).approx_eq(&F32(1.00005), 0.0001)` → `true`
- Floats outside tolerance: `F32(1.0).approx_eq(&F32(1.0002), 0.0001)` → `false`
- Booleans: `Bool(true).approx_eq(&Bool(true), 0.1)` → `true` (exact match required)
- Vectors within tolerance: `Vec2([1.0, 2.0]).approx_eq(&Vec2([1.00005, 2.00005]), 0.0001)` → `true`
- Vectors outside tolerance: `Vec2([1.0, 2.0]).approx_eq(&Vec2([1.0002, 2.0]), 0.0001)` → `false`
- Matrices: `Mat2x2([[1.0, 2.0], [3.0, 4.0]]).approx_eq(&Mat2x2([[1.00005, 2.0], [3.0, 4.0]]), 0.0001)` → `true`
- Type mismatch: `I32(42).approx_eq(&F32(42.0), 0.1)` → `false`

**Default tolerance (`approx_eq_default()`):**

- `F32(1.0).approx_eq_default(&F32(1.00005))` → `true` (within 1e-4)
- `F32(1.0).approx_eq_default(&F32(1.0002))` → `false` (outside 1e-4)

## Implementation Steps

1. **Implement `GlslValue::parse()`:**

   - Add method to `GlslValue` impl in `executable.rs`
   - Use GLSL parser to parse literal string
   - Type check to ensure it's a literal constant
   - Convert to appropriate `GlslValue` variant
   - Add unit tests

2. **Implement `parse_program_with_registry()`:**

   - Create function in appropriate module (likely `pipeline.rs` or new helper)
   - Parse GLSL source using existing parser
   - Run semantic analysis
   - Extract `FunctionRegistry` from semantic result
   - Add unit tests

3. **Verify/Enhance `infer_expr_type_in_context()`:**

   - Check if function already exists
   - If not, create wrapper around `infer_expr_type_with_registry()`
   - Parse expression string first
   - Use function registry for type inference
   - Add unit tests

4. **Implement `GlslValue` comparison operators:**

   - Add `eq()` method for exact equality
   - Add `approx_eq()` method for approximate equality with tolerance
   - Add `approx_eq_default()` convenience method
   - Handle all `GlslValue` variants (I32, F32, Bool, Vec2/3/4, Mat2x2/3x3/4x4)
   - Add comprehensive unit tests covering all cases

## Success Criteria

- [ ] `GlslValue::parse()` implemented with unit tests
- [ ] `parse_program_with_registry()` implemented with unit tests
- [ ] `infer_expr_type_in_context()` implemented/verified with unit tests
- [ ] `GlslValue::eq()` implemented with unit tests
- [ ] `GlslValue::approx_eq()` implemented with unit tests
- [ ] `GlslValue::approx_eq_default()` implemented with unit tests
- [ ] All unit tests pass
- [ ] Functions are exported/public for use by `lp-glsl-filetests`
