# Testing Guide for LightPlayer Cranelift

## Overview

This document outlines the testing strategy and guidelines for LightPlayer Cranelift code, particularly for `lp-glsl`. The guide is based on patterns observed in the Cranelift codebase and emphasizes clarity, maintainability, and comprehensive test coverage.

## Testing Philosophy

**Most testing should be done in filetests** (`lp-glsl-filetests`) because they are:

- **Terse**: Easy to read and write
- **Powerful**: Can test full compilation pipelines, execution, and IR transformations
- **Maintainable**: Test data is separate from test logic
- **Comprehensive**: Can test complex interactions between compiler passes

**Some unit tests are important** for:

- **Quick spot checks**: `cargo test` can quickly verify basic functionality
- **Pure functions**: Testing isolated logic without full compilation
- **Error handling**: Testing error types and messages
- **Edge cases**: Testing boundary conditions in small, focused tests

## Test Organization

### Filetests Structure

Filetests are organized in `crates/lp-glsl-filetests/filetests/` by feature domain:

```
filetests/
├── basic/              # Basic language features (literals, arithmetic)
├── control_flow/       # Control structures (if/else, loops)
├── builtins/           # Built-in functions (mix, step, sin, etc.)
├── vectors/            # Vector operations
├── matrices/           # Matrix operations
├── functions/          # User-defined functions
├── fixed32/            # Fixed-point transformation tests
├── float/              # Floating-point operations
├── type_errors/        # Type checking error tests
└── ...
```

Each test file is a `.glsl` file containing:

1. Test directives (what to test)
2. Target specifications (where to run)
3. GLSL source code
4. Expected outputs (CLIF IR, execution results, or errors)

### Unit Tests Structure

Unit tests live in `#[cfg(test)]` modules within the source files they test. They should be:

- **Co-located**: In the same file as the code being tested
- **Focused**: Test a single function or small set of related functions
- **Fast**: Quick to run, no compilation overhead

## When to Write Unit Tests vs Filetests

### Write Unit Tests For:

1. **Pure utility functions**

   - String manipulation, parsing helpers
   - Type conversion utilities
   - Mathematical helpers (not involving IR)

   Example: Testing `SourceLocation::display()` formatting

2. **Error types and messages**

   - Error code enums
   - Error message formatting
   - Error location tracking

   Example: Testing `GlslError::undefined_variable()` creates correct error

3. **Data structure operations**

   - Collections, maps, sets
   - Simple transformations on data structures
   - Validation logic

   Example: Testing symbol table insertion/lookup

4. **Edge cases in small functions**

   - Boundary conditions
   - Special values (zero, max, min)
   - Empty inputs

   Example: Testing division by zero detection logic

### Write Filetests For:

1. **GLSL compilation**

   - Full compilation pipeline from GLSL → CLIF → binary
   - IR generation correctness
   - Code generation for different targets

2. **Compiler passes and transformations**

   - Fixed-point transformations (`fixed32`, `fixed64`)
   - Optimizations
   - Legalization
   - Any multi-pass interaction

3. **Execution correctness**

   - End-to-end behavior: GLSL → execution → results
   - Built-in function correctness
   - Arithmetic operations
   - Control flow behavior

4. **Type checking**

   - Type inference
   - Type errors (use `test error` directive)
   - Type conversions

5. **Language features**

   - Vectors, matrices
   - Functions
   - Control flow
   - Operators

6. **Target-specific behavior**
   - RISC-V 32-bit code generation
   - Fixed-point arithmetic on different targets
   - Architecture-specific optimizations

## Filetest Commands

### `test compile`

Tests that GLSL code compiles to expected CLIF IR. Use this to verify:

- IR generation correctness
- Instruction selection
- Code structure

```glsl
// test compile
// target riscv32

int main() {
    return 42;
}

// CHECK: function u0:0() -> i32
// CHECK: block0:
// CHECK:     v0 = iconst.i32 42
// CHECK:     return v0
```

### `test run`

Tests that compiled code executes correctly. Use this to verify:

- Execution correctness
- Arithmetic operations
- Built-in functions
- End-to-end behavior

```glsl
// test run
// target host

int main() {
    return 2 + 3;
}

// run: == 5
```

### `test error`

Tests that compilation produces expected error messages. Use this to verify:

- Type checking errors
- Syntax errors
- Semantic errors

```glsl
// test error

int main() {
    float x = 10;
    int y = x;  // EXPECT_ERROR: type mismatch
    return y;
}
```

### `test riscv32.fixed32`

Tests fixed-point transformation for RISC-V 32-bit target. This is a special test that:

- Generates CLIF before transformation
- Generates CLIF after fixed-point transformation
- Executes code in RISC-V 32-bit emulator
- Verifies both IR and execution

```glsl
// test riscv32.fixed32
// test run

float main() {
    return 2.5 + 3.5;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.400000p1
//     v1 = f32const 0x1.c00000p1
//     v2 = fadd v0, v1
//     return v2
// }

// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0002_8000
//     v1 = iconst.i32 0x0003_8000
//     v2 = iadd v0, v1
//     return v2
// }
// run: ≈ 6.0
```

### Multiple Test Commands

You can combine multiple test commands in a single file:

```glsl
// test compile
// test run
// target host
// target riscv32

int main() {
    return 10 + 20;
}

// CHECK: v0 = iconst.i32 10
// CHECK: v1 = iconst.i32 20
// CHECK: v2 = iadd v0, v1
// run: == 30
```

This runs both compilation checks and execution tests on multiple targets.

## Target Specifications

### Host Target

```glsl
// target host
```

Runs tests on the host machine (native execution).

### RISC-V 32-bit Target

```glsl
// target riscv32
```

Runs tests using RISC-V 32-bit emulator.

### Fixed-Point Targets

```glsl
// target host.fixed32
// target riscv32.fixed32
```

Runs tests with fixed-point transformation (Fixed16x16 format).

```glsl
// target host.fixed64
// target riscv32.fixed64
```

Runs tests with fixed-point transformation (Fixed32x32 format). Note: Fixed64 targets may be skipped if 128-bit operations aren't supported.

### Multiple Targets

You can specify multiple targets, and the test will run on all of them:

```glsl
// test run
// target host
// target riscv32
// target host.fixed32
// target riscv32.fixed32

int main() {
    return 42;
}

// run: == 42
```

## Run Directives

### Exact Match

```glsl
// run: == 42
```

### Approximate Match (for floating-point)

```glsl
// run: ≈ 3.14159
```

### Function Calls with Arguments

```glsl
int add(int a, int b) {
    return a + b;
}

int main() {
    return add(10, 20);
}

// run: == 30
```

### Multiple Run Directives

```glsl
int main() {
    return 42;
}

// run: == 42
// run: == 42  // Can have duplicates for clarity
```

## Filetest Best Practices

### 1. Test One Concept Per File

Each test file should focus on testing one specific feature or behavior:

```
✅ Good: filetests/basic/int_literal.glsl
✅ Good: filetests/builtins/mix_scalar.glsl
❌ Bad: filetests/basic/everything.glsl
```

### 2. Use Descriptive File Names

File names should clearly indicate what is being tested:

```
✅ Good:
- int_literal.glsl
- vec3_add.glsl
- fixed32_mul.glsl
- type_error_undefined_variable.glsl

❌ Bad:
- test1.glsl
- stuff.glsl
- foo.glsl
```

### 3. Group Related Tests

Organize tests by feature domain in subdirectories:

```
filetests/
├── builtins/
│   ├── interpolation/
│   │   ├── mix_scalar.glsl
│   │   └── step_scalar.glsl
│   └── geometric/
│       ├── dot_vec3.glsl
│       └── cross_vec3.glsl
```

### 4. Include Both Positive and Negative Tests

Test both correct behavior and error cases:

```
✅ Good:
- filetests/basic/int_literal.glsl (positive)
- filetests/type_errors/int_condition.glsl (negative)
```

### 5. Test Edge Cases

Include tests for boundary conditions, special values, and edge cases:

```glsl
// test run
// target host

int main() {
    int x = 2147483647;  // Max i32
    return x;
}

// run: == 2147483647
```

### 6. Use CHECK Directives Sparingly

Only include CHECK directives when testing specific IR patterns. For execution tests, prefer `run:` directives:

```glsl
// test compile
// test run

int main() {
    return 2 + 3;
}

// Prefer this:
// run: == 5

// Over this (unless testing specific IR):
// CHECK: v0 = iconst.i32 2
// CHECK: v1 = iconst.i32 3
// CHECK: v2 = iadd v0, v1
```

### 7. Test Transformations Explicitly

For transformation tests (like fixed32), show both before and after:

```glsl
// test riscv32.fixed32

float main() {
    return 2.5 + 3.5;
}

// Generated CLIF (before transformation)
// ... show original CLIF ...

// Transformed CLIF (after transformation)
// ... show transformed CLIF ...

// run: ≈ 6.0
```

## Unit Test Best Practices

### 1. Keep Tests Focused

Each test should verify one specific behavior:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(5, 10);
        assert_eq!(loc.to_string(), "5:10");
    }

    #[test]
    fn test_source_location_with_file() {
        let loc = SourceLocation::with_file(5, 10, "shader.glsl".into());
        assert_eq!(loc.to_string(), "shader.glsl:5:10");
    }
}
```

### 2. Use Descriptive Test Names

Test names should clearly describe what is being tested:

```rust
✅ Good:
- test_source_location_display
- test_error_code_display
- test_glsl_error_with_location

❌ Bad:
- test1
- test_error
- test_thing
```

### 3. Test Error Cases

Don't just test the happy path:

```rust
#[test]
fn test_glsl_error_undefined_variable() {
    let err = GlslError::undefined_variable("foo");
    assert_eq!(err.code, ErrorCode::E0100);
    assert!(err.message.contains("foo"));
}

#[test]
fn test_glsl_error_with_location() {
    let err = GlslError::undefined_variable("foo")
        .with_location(SourceLocation::new(5, 10));

    let display = err.to_string();
    assert!(display.contains("E0100"));
    assert!(display.contains("5:10"));
}
```

### 4. Use Appropriate Assertions

Choose assertions that provide good error messages:

```rust
✅ Good:
assert_eq!(result, expected);
assert!(value.contains("substring"));
assert!(value.is_empty());

❌ Bad:
assert!(result == expected);  // Less informative error message
```

## Example: Testing a New Feature (Arrays)

This example shows how to test a hypothetical arrays feature in `lp-glsl`.

### Step 1: Unit Tests for Core Logic

First, write unit tests for any pure functions or data structures:

```rust
// crates/lp-glsl/src/types.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_type_creation() {
        let elem_type = Type::Int;
        let array_type = ArrayType::new(elem_type, 10);
        assert_eq!(array_type.element_type(), Type::Int);
        assert_eq!(array_type.length(), 10);
    }

    #[test]
    fn test_array_type_equality() {
        let arr1 = ArrayType::new(Type::Int, 10);
        let arr2 = ArrayType::new(Type::Int, 10);
        let arr3 = ArrayType::new(Type::Float, 10);

        assert_eq!(arr1, arr2);
        assert_ne!(arr1, arr3);
    }
}
```

### Step 2: Filetests for Compilation

Test that arrays compile correctly:

```glsl
// filetests/arrays/declaration.glsl
// test compile
// target host

int main() {
    int arr[5];
    return 0;
}

// CHECK: function u0:0() -> i32
// CHECK: block0:
// CHECK:     ; array declaration: arr[5]
```

### Step 3: Filetests for Execution

Test that array operations execute correctly:

```glsl
// filetests/arrays/indexing.glsl
// test run
// target host

int main() {
    int arr[3];
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    return arr[1];
}

// run: == 20
```

### Step 4: Filetests for Type Errors

Test that type errors are caught:

```glsl
// filetests/type_errors/array_index_type.glsl
// test error

int main() {
    int arr[5];
    float idx = 2.0;
    return arr[idx];  // EXPECT_ERROR: array index must be integer
}
```

### Step 5: Filetests for Fixed-Point Transformation

Test that arrays work with fixed-point transformation:

```glsl
// filetests/fixed32/array.glsl
// test riscv32.fixed32
// test run

float main() {
    float arr[3];
    arr[0] = 1.5;
    arr[1] = 2.5;
    arr[2] = 3.5;
    return arr[1];
}

// Generated CLIF
// ... show CLIF with f32 array operations ...

// Transformed CLIF
// ... show CLIF with i32 array operations (fixed-point) ...

// run: ≈ 2.5
```

### Step 6: Filetests for Edge Cases

Test boundary conditions and edge cases:

```glsl
// filetests/arrays/bounds_check.glsl
// test error

int main() {
    int arr[5];
    return arr[5];  // EXPECT_ERROR: array index out of bounds
}

// filetests/arrays/zero_length.glsl
// test error

int main() {
    int arr[0];  // EXPECT_ERROR: array length must be > 0
    return 0;
}
```

### Step 7: Integration Tests

Test arrays with other features:

```glsl
// filetests/arrays/with_functions.glsl
// test run
// target host

int get_element(int arr[5], int idx) {
    return arr[idx];
}

int main() {
    int arr[5];
    arr[2] = 42;
    return get_element(arr, 2);
}

// run: == 42
```

## Running Tests

### Run All Filetests

```bash
cargo test -p lp-glsl-filetests
```

### Run Specific Filetest

```bash
cargo test -p lp-glsl-filetests test_int_literal
```

### Run Unit Tests

```bash
cargo test -p lp-glsl
```

### Run Tests with Output

```bash
cargo test -p lp-glsl-filetests -- --nocapture
```

### Update Test Expectations (BLESS Mode)

When CLIF output changes intentionally, update expectations:

```bash
CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests test_name
```

This automatically updates the expected CLIF in test files.

## Test Maintenance

### When Tests Fail

1. **Understand the failure**: Read the error message carefully
2. **Check if it's expected**: Did you intentionally change behavior?
3. **Update expectations**: Use BLESS mode if the change is correct
4. **Fix the bug**: If the failure indicates a real bug, fix it

### Adding New Tests

1. **Create test file**: Add `.glsl` file in appropriate directory
2. **Add test function**: Add test function to `tests/filetests.rs`
3. **Run test**: Verify it passes (or fails as expected)
4. **Commit**: Include test with feature implementation

### Test Organization

- **Keep tests focused**: One concept per test file
- **Group related tests**: Use subdirectories for organization
- **Name descriptively**: File names should indicate what's tested
- **Document complex tests**: Add comments explaining non-obvious test cases

## Common Patterns

### Testing Arithmetic Operations

```glsl
// test run
// target host

int main() {
    return 10 + 20;
}

// run: == 30
```

### Testing Built-in Functions

```glsl
// test run
// target host

float main() {
    return mix(0.0, 1.0, 0.5);
}

// run: ≈ 0.5
```

### Testing Vectors

```glsl
// test run
// target host

vec3 main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    return a + b;
}

// run: ≈ vec3(5.0, 7.0, 9.0)
```

### Testing Fixed-Point Transformation

```glsl
// test riscv32.fixed32
// test run

float main() {
    return 2.5 * 3.0;
}

// Generated CLIF
// ... before transformation ...

// Transformed CLIF
// ... after transformation ...

// run: ≈ 7.5
```

### Testing Type Errors

```glsl
// test error

int main() {
    float x = 10.0;
    int y = x;  // EXPECT_ERROR: cannot convert float to int
    return y;
}
```

## Summary

### ✅ Do

- **Write filetests for most features**: They're terse and powerful
- **Write unit tests for pure functions**: Quick spot checks
- **Test both positive and negative cases**: Correct behavior and errors
- **Test edge cases**: Boundaries, special values
- **Organize tests by feature**: Use descriptive directories and file names
- **Use multiple test commands**: Combine `compile`, `run`, `error` as needed
- **Test transformations explicitly**: Show before/after for fixed-point

### ❌ Don't

- **Don't write unit tests for full compilation**: Use filetests instead
- **Don't test implementation details unnecessarily**: Test behavior, not IR structure
- **Don't create overly complex test files**: One concept per file
- **Don't skip error tests**: Test that errors are caught correctly
- **Don't forget edge cases**: Test boundaries and special values
- **Don't use generic test names**: Be descriptive

## References

- [Cranelift Testing Documentation](../cranelift/docs/testing.md)
- [Cranelift Filetests README](../cranelift/filetests/README.md)
- [Rust Book: Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
