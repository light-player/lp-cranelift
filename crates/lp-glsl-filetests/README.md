# lp-glsl-filetests

Cranelift-style filetest infrastructure for validating GLSL compilation.

## Test Types

- `test compile` - Validates Cranelift IR generation against expected output
- `test run` - Executes JIT-compiled code and verifies results
- `test error` - Validates that compilation fails with expected error
- `test fixed16` - Use 16.16 fixed-point format
- `test fixed32` - Use 32.32 fixed-point format

## Test File Format

Test files use GLSL's native comment syntax with expectations embedded as trailing comments:

```glsl
// test compile
// test run

int main() {
    int a = 10;
    int b = 20;
    return a + b;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 20
//     v2 = iadd v0, v1  ; v0 = 10, v1 = 20
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: == 30
```

### Expectation Directives

- `// <clif-output>` - Expected CLIF IR (for `test compile`)
- `// run: == <value>` - Verify execution result equals value (int or bool)
- `// run: ~= <value> (tolerance: <tol>)` - Verify float result within tolerance
- `// EXPECT_ERROR: <pattern>` - Verify error message contains pattern

## BLESS Mode

Automatically update test expectations using the `CRANELIFT_TEST_BLESS=1` environment variable:

```bash
# Update all test expectations
CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests

# Update specific test
CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests test_arithmetic
```

BLESS mode updates:
- Compile test expectations with actual CLIF output
- Run directives with actual execution results
- Error patterns with actual error messages

Use BLESS when adding new tests or after intentional codegen changes. Always review the generated output to ensure correctness.

## Running Tests

```bash
# Run all tests
cargo test -p lp-glsl-filetests

# Run specific test
cargo test -p lp-glsl-filetests test_arithmetic

# Show output
cargo test -p lp-glsl-filetests -- --nocapture
```

## Test Organization

- `basic/` - Basic language features (literals, arithmetic, comparisons)
- `control_flow/` - Control structures (if/else, loops, break/continue)
- `builtins/` - Built-in functions (mix, step, smoothstep, etc.)
- `vectors/` - Vector operations (construction, arithmetic, components, swizzles)
- `float/` - Floating-point operations
- `fixed_point/` - Fixed-point transformation tests
- `functions/` - User-defined functions
- `type_errors/` - Type checking error tests

## Adding New Tests

1. Create a `.glsl` file with test directives and GLSL code
2. Run with BLESS to generate expectations: `CRANELIFT_TEST_BLESS=1 cargo test -p lp-glsl-filetests test_name`
3. Review the generated expectations
4. Add the test function to `tests/filetests.rs`
