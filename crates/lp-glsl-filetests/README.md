# lp-glsl-filetests

Cranelift-style filetest infrastructure for validating GLSL compilation.

## Overview

This crate provides two types of tests for lp-glsl:

1. **`test compile`** - Validates Cranelift IR generation with CHECK directives
2. **`test run`** - Executes JIT-compiled code and verifies results

## Test File Format

Test files use GLSL's native `//` comment syntax:

```glsl
// test compile
// test run

int main() {
    int a = 10;
    int b = 20;
    return a + b;
}

// CHECK: iconst.i32 10
// CHECK: iconst.i32 20
// CHECK: iadd
// run: == 30
```

### Test Directives

- `// test compile` - Enable CLIF IR verification
- `// test run` - Enable JIT execution verification
- `// CHECK: <pattern>` - Verify pattern appears in generated CLIF IR
- `// run: == <value>` - Verify execution result (int or bool)

## Running Tests

```bash
# Run all filetests
cargo test -p lp-glsl-filetests

# Run specific test
cargo test -p lp-glsl-filetests test_arithmetic

# Show detailed output
cargo test -p lp-glsl-filetests -- --nocapture
```

## Test Coverage

Phase 1 filetests in `filetests/basic/`:

- `int_literal.glsl` - Integer literals
- `bool_literal.glsl` - Boolean literals
- `arithmetic.glsl` - Integer arithmetic (+, -, *, /)
- `comparisons.glsl` - Integer comparisons (<, >, ==, !=, <=, >=)
- `unary.glsl` - Unary operators (-, !)
- `complex_expr.glsl` - Complex expressions
- `assignment.glsl` - Variable assignments
- `bool_not.glsl` - Boolean NOT operator

All 8 tests verify both CLIF IR generation and runtime execution.

## Benefits

1. **Valid GLSL** - Test files are valid GLSL code using `//` comments
2. **Dual verification** - Both IR and execution correctness
3. **Clear examples** - Tests show expected IR and results inline
4. **Easy debugging** - Separate compile and run failures
5. **Regression prevention** - Catch codegen and runtime issues early

