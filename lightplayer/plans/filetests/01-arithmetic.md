# Arithmetic Operations Test Plan

## Overview

This plan defines a comprehensive test suite for GLSL arithmetic operations using the filetest infrastructure in `lightplayer/crates/lp-glsl-filetests`. Tests are organized by operation type (add, subtract, multiply, divide, modulo) and then by data type within each file.

## Reference

- **GLSL Spec**: `/Users/yona/dev/photomancer/glsl-spec/chapters/operators.adoc`
- **Key Sections**:
  - Arithmetic binary operators (lines 774-831): `+`, `-`, `*`, `/`
  - Modulus operator (lines 832-855): `%` (integer only)
  - Vector and matrix operations (lines 995-1100)

## Test Organization

Tests are organized in `filetests/math/` with the following structure (type-first naming):

```
filetests/math/
├── float-add.glsl      # Addition for float, vec2, vec3, vec4, mat2, mat3, mat4
├── int-add.glsl        # Addition for int, ivec2, ivec3, ivec4
├── float-subtract.glsl # Subtraction for float, vec2, vec3, vec4, mat2, mat3, mat4
├── int-subtract.glsl   # Subtraction for int, ivec2, ivec3, ivec4
├── float-multiply.glsl # Multiplication for float, vec2, vec3, vec4, mat2, mat3, mat4
├── int-multiply.glsl   # Multiplication for int, ivec2, ivec3, ivec4
├── float-divide.glsl   # Division for float, vec2, vec3, vec4, mat2, mat3, mat4
├── int-divide.glsl     # Division for int, ivec2, ivec3, ivec4
└── int-modulo.glsl     # Modulo for int, ivec2, ivec3, ivec4 (integer only)
```

## Test File Structure

Each test file follows this pattern:

1. **Directives** at the top:

   - `// test compile` - Verify CLIF IR generation
   - `// test transform.fixed32` - Verify fixed32 transformation
   - `// test run` - Execute and verify results
   - `// target riscv32.fixed32` - Target specification

2. **GLSL Functions** organized by data type:

   - Scalar functions first (e.g., `float`, `int`)
   - Vector functions (e.g., `vec2`, `vec3`, `vec4`, `ivec2`, `ivec3`, `ivec4`)
   - Matrix functions (e.g., `mat2`, `mat3`, `mat4`) - float only

3. **CLIF Expectations** (generated via bless mode):

   - Pre-transform CLIF for `// #compile: initial.clif`
   - Post-transform CLIF for `// #transform: fixed32.clif`

4. **Run Directives** (`// #run:`) for each function:
   - Multiple `// #run:` tests per function covering edge cases
   - Use `~=` for float comparisons (with tolerance)
   - Use `==` for integer comparisons (exact)
   - Format: `// #run: function_name(args) == expected` or `// #run: function_name(args) ~= expected`

## GLSL Arithmetic Operation Rules

Based on the GLSL spec (operators.adoc, lines 774-831):

### Valid Operation Combinations

1. **Scalar + Scalar** → Scalar

   - `float + float` → `float`
   - `int + int` → `int`

2. **Scalar + Vector/Matrix** → Vector/Matrix (component-wise)

   - `float + vec3` → `vec3` (scalar applied to each component)
   - `float + mat2` → `mat2` (scalar applied to each component)

3. **Vector + Vector** → Vector (component-wise, same size)

   - `vec2 + vec2` → `vec2`
   - `vec3 + vec3` → `vec3`
   - `vec4 + vec4` → `vec4`

4. **Matrix + Matrix** → Matrix (component-wise, same dimensions)

   - `mat2 + mat2` → `mat2` (for `+`, `-`, `/`)
   - `mat3 + mat3` → `mat3`
   - `mat4 + mat4` → `mat4`

5. **Matrix Multiplication** (special case for `*`):
   - `mat2 * mat2` → `mat2` (linear algebraic multiply)
   - `vec2 * mat2` → `vec2` (vector treated as row vector)
   - `mat2 * vec2` → `vec2` (vector treated as column vector)
   - Matrix dimensions must match: columns of left = rows of right

### Modulo Operator (`%`)

- **Integer only**: `int`, `ivec2`, `ivec3`, `ivec4`
- **Not valid**: `float`, `vec2`, `vec3`, `vec4`, matrices
- Component-wise for vectors
- Undefined behavior: division by zero, negative operands

## Implementation Note

**Vector/Matrix Parsing**: The current `parse_glsl_value()` function in `test_run.rs` only supports scalar types (i32, f32, bool). To support vector and matrix test expectations, we need to:

1. Update `parse_glsl_value()` to parse vector constructors like `vec2(1.0, 2.0)` and matrix constructors like `mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))`
2. Update `execute_main()` to handle vector and matrix return types
3. Update `compare_results()` to use `GlslValue::approx_eq()` for vector/matrix comparisons

This should be done in `lightplayer/crates/lp-glsl/src/backend/glsl_value.rs` and `lightplayer/crates/lp-glsl-filetests/src/test_run.rs`.

## Test Cases by File

### 1. `float-add.glsl`

**Operations**: Addition (`+`) for floating-point types

**Functions to test**:

1. **Scalar**:

   ```glsl
   float add_float(float a, float b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests covering edge cases:

   ```glsl
   // #run: add_float(0.0, 0.0) ~= 0.0
   // #run: add_float(1.5, 2.5) ~= 4.0
   // #run: add_float(-1.5, 2.5) ~= 1.0
   // #run: add_float(1.5, -2.5) ~= -1.0
   // #run: add_float(-1.5, -2.5) ~= -4.0
   // #run: add_float(100.0, 200.0) ~= 300.0
   // #run: add_float(0.001, 0.002) ~= 0.003
   // #run: add_float(1e10, 1e10) ~= 2e10
   ```

2. **vec2**:

   ```glsl
   vec2 add_vec2(vec2 a, vec2 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: add_vec2(vec2(0.0, 0.0), vec2(0.0, 0.0)) ~= vec2(0.0, 0.0)
   // #run: add_vec2(vec2(1.0, 2.0), vec2(3.0, 4.0)) ~= vec2(4.0, 6.0)
   // #run: add_vec2(vec2(-1.0, 2.0), vec2(3.0, -4.0)) ~= vec2(2.0, -2.0)
   // #run: add_vec2(vec2(100.0, 200.0), vec2(50.0, 75.0)) ~= vec2(150.0, 275.0)
   ```

3. **vec3**:

   ```glsl
   vec3 add_vec3(vec3 a, vec3 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: add_vec3(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)) ~= vec3(0.0, 0.0, 0.0)
   // #run: add_vec3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0)) ~= vec3(5.0, 7.0, 9.0)
   // #run: add_vec3(vec3(-1.0, 2.0, -3.0), vec3(4.0, -5.0, 6.0)) ~= vec3(3.0, -3.0, 3.0)
   ```

4. **vec4**:

   ```glsl
   vec4 add_vec4(vec4 a, vec4 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: add_vec4(vec4(0.0, 0.0, 0.0, 0.0), vec4(0.0, 0.0, 0.0, 0.0)) ~= vec4(0.0, 0.0, 0.0, 0.0)
   // #run: add_vec4(vec4(1.0, 2.0, 3.0, 4.0), vec4(5.0, 6.0, 7.0, 8.0)) ~= vec4(6.0, 8.0, 10.0, 12.0)
   // #run: add_vec4(vec4(-1.0, 2.0, -3.0, 4.0), vec4(5.0, -6.0, 7.0, -8.0)) ~= vec4(4.0, -4.0, 4.0, -4.0)
   ```

5. **Scalar + Vector** (component-wise):

   ```glsl
   vec2 add_scalar_vec2(float s, vec2 v) {
       return s + v;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: add_scalar_vec2(0.0, vec2(0.0, 0.0)) ~= vec2(0.0, 0.0)
   // #run: add_scalar_vec2(2.0, vec2(1.0, 3.0)) ~= vec2(3.0, 5.0)
   // #run: add_scalar_vec2(-1.0, vec2(2.0, 4.0)) ~= vec2(1.0, 3.0)
   // #run: add_scalar_vec2(100.0, vec2(-50.0, 50.0)) ~= vec2(50.0, 150.0)
   ```

6. **mat2**:

   ```glsl
   mat2 add_mat2(mat2 a, mat2 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests (component-wise addition, column-major order):

   ```glsl
   // #run: add_mat2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), mat2(vec2(5.0, 6.0), vec2(7.0, 8.0))) ~= mat2(vec2(6.0, 8.0), vec2(10.0, 12.0))
   // #run: add_mat2(mat2(vec2(0.0, 0.0), vec2(0.0, 0.0)), mat2(vec2(0.0, 0.0), vec2(0.0, 0.0))) ~= mat2(vec2(0.0, 0.0), vec2(0.0, 0.0))
   // #run: add_mat2(mat2(vec2(-1.0, 2.0), vec2(3.0, -4.0)), mat2(vec2(5.0, -6.0), vec2(-7.0, 8.0))) ~= mat2(vec2(4.0, -4.0), vec2(-4.0, 4.0))
   ```

7. **mat3**:

   ```glsl
   mat3 add_mat3(mat3 a, mat3 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests (similar pattern to mat2)

8. **mat4**:
   ```glsl
   mat4 add_mat4(mat4 a, mat4 b) {
       return a + b;
   }
   ```
   - Multiple `// #run:` tests (similar pattern to mat2)

### 2. `int-add.glsl`

**Operations**: Addition (`+`) for integer types

**Functions to test**:

1. **Scalar**:

   ```glsl
   int add_int(int a, int b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: add_int(0, 0) == 0
   // #run: add_int(1, 2) == 3
   // #run: add_int(-1, -2) == -3
   // #run: add_int(5, -3) == 2
   // #run: add_int(-5, 3) == -2
   // #run: add_int(1000, 2000) == 3000
   // #run: add_int(2147483647, 0) == 2147483647
   // #run: add_int(-2147483648, 0) == -2147483648
   ```

2. **ivec2**:

   ```glsl
   ivec2 add_ivec2(ivec2 a, ivec2 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: add_ivec2(ivec2(0, 0), ivec2(0, 0)) == ivec2(0, 0)
   // #run: add_ivec2(ivec2(1, 2), ivec2(3, 4)) == ivec2(4, 6)
   // #run: add_ivec2(ivec2(-1, 2), ivec2(3, -4)) == ivec2(2, -2)
   // #run: add_ivec2(ivec2(1000, 2000), ivec2(500, 750)) == ivec2(1500, 2750)
   ```

3. **ivec3**:

   ```glsl
   ivec3 add_ivec3(ivec3 a, ivec3 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests (similar pattern)

4. **ivec4**:

   ```glsl
   ivec4 add_ivec4(ivec4 a, ivec4 b) {
       return a + b;
   }
   ```

   - Multiple `// #run:` tests (similar pattern)

5. **Scalar + Vector**:
   ```glsl
   ivec2 add_scalar_ivec2(int s, ivec2 v) {
       return s + v;
   }
   ```
   - Multiple `// #run:` tests:
   ```glsl
   // #run: add_scalar_ivec2(0, ivec2(0, 0)) == ivec2(0, 0)
   // #run: add_scalar_ivec2(2, ivec2(1, 3)) == ivec2(3, 5)
   // #run: add_scalar_ivec2(-1, ivec2(2, 4)) == ivec2(1, 3)
   ```

**Note**: Use `==` for exact equality (no tolerance needed for integers)

### 3. `float-subtract.glsl`

**Operations**: Subtraction (`-`) for floating-point types

**Functions**: Same structure as `float-add.glsl` but using subtraction operator

**Example test cases for scalar**:

```glsl
// #run: subtract_float(0.0, 0.0) ~= 0.0
// #run: subtract_float(5.0, 2.0) ~= 3.0
// #run: subtract_float(-5.0, -2.0) ~= -3.0
// #run: subtract_float(5.0, -2.0) ~= 7.0
// #run: subtract_float(2.0, 5.0) ~= -3.0
// #run: subtract_float(1.0001, 1.0) ~= 0.0001
// #run: subtract_float(100.0, 50.0) ~= 50.0
```

### 4. `int-subtract.glsl`

**Operations**: Subtraction (`-`) for integer types

**Functions**: Same structure as `int-add.glsl` but using subtraction operator

**Example test cases for scalar**:

```glsl
// #run: subtract_int(0, 0) == 0
// #run: subtract_int(5, 2) == 3
// #run: subtract_int(-5, -2) == -3
// #run: subtract_int(5, -2) == 7
// #run: subtract_int(2, 5) == -3
// #run: subtract_int(1000, 500) == 500
```

### 5. `float-multiply.glsl`

**Operations**: Multiplication (`*`) for floating-point types

**Functions to test**:

1. **Scalar × Scalar**:

   ```glsl
   float mul_float(float a, float b) {
       return a * b;
   }
   ```

2. **Vector × Vector** (component-wise):

   ```glsl
   vec2 mul_vec2(vec2 a, vec2 b) {
       return a * b;
   }
   ```

   - Multiple `// #run:` tests

3. **Scalar × Vector**:

   ```glsl
   vec2 mul_scalar_vec2(float s, vec2 v) {
       return s * v;
   }
   ```

   - Multiple `// #run:` tests

4. **Matrix × Matrix** (linear algebraic):

   ```glsl
   mat2 mul_mat2(mat2 a, mat2 b) {
       return a * b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: mul_mat2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), mat2(vec2(5.0, 6.0), vec2(7.0, 8.0))) ~= mat2(vec2(19.0, 22.0), vec2(43.0, 50.0))
   // Formula: result[i][j] = sum(a[i][k] * b[k][j])
   ```

5. **Vector × Matrix** (row vector × matrix):

   ```glsl
   vec2 mul_vec2_mat2(vec2 v, mat2 m) {
       return v * m;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: mul_vec2_mat2(vec2(1.0, 2.0), mat2(vec2(3.0, 4.0), vec2(5.0, 6.0))) ~= vec2(13.0, 16.0)
   // Result: [1*3+2*5, 1*4+2*6] = [13, 16]
   ```

6. **Matrix × Vector** (matrix × column vector):
   ```glsl
   vec2 mul_mat2_vec2(mat2 m, vec2 v) {
       return m * v;
   }
   ```
   - Multiple `// #run:` tests:
   ```glsl
   // #run: mul_mat2_vec2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), vec2(5.0, 6.0)) ~= vec2(17.0, 39.0)
   // Result: [1*5+2*6, 3*5+4*6] = [17, 39]
   ```

### 6. `multiply-int.glsl`

**Operations**: Multiplication (`*`) for integer types

**Functions**: Same structure as `int-add.glsl` but using multiplication operator

**Note**: No matrix multiplication for integer types (matrices are float-only)

**Example test cases for scalar**:

```glsl
// #run: mul_int(0, 5) == 0
// #run: mul_int(1, 5) == 5
// #run: mul_int(2, 3) == 6
// #run: mul_int(-2, 3) == -6
// #run: mul_int(-2, -3) == 6
// #run: mul_int(100, 200) == 20000
```

### 7. `float-divide.glsl`

**Operations**: Division (`/`) for floating-point types

**Functions**: Same structure as `float-add.glsl` but using division operator

**Example test cases for scalar**:

```glsl
// #run: divide_float(5.0, 1.0) ~= 5.0
// #run: divide_float(10.0, 2.0) ~= 5.0
// #run: divide_float(-10.0, 2.0) ~= -5.0
// #run: divide_float(-10.0, -2.0) ~= 5.0
// #run: divide_float(1.0, 2.0) ~= 0.5
// #run: divide_float(0.1, 0.2) ~= 0.5
// #run: divide_float(1000.0, 10.0) ~= 100.0
```

**Note**: Division by zero is undefined behavior (not tested)

### 8. `int-divide.glsl`

**Operations**: Division (`/`) for integer types

**Functions**: Same structure as `int-add.glsl` but using division operator

**Example test cases for scalar**:

```glsl
// #run: divide_int(10, 2) == 5
// #run: divide_int(-10, 2) == -5
// #run: divide_int(-10, -2) == 5
// #run: divide_int(7, 2) == 3
// #run: divide_int(1000, 10) == 100
```

**Note**: Integer division truncates toward zero. Division by zero is undefined.

### 9. `int-modulo.glsl`

**Operations**: Modulo (`%`) for integer types only

**Functions to test**:

1. **Scalar**:

   ```glsl
   int mod_int(int a, int b) {
       return a % b;
   }
   ```

   - Multiple `// #run:` tests:

   ```glsl
   // #run: mod_int(10, 3) == 1
   // #run: mod_int(10, 5) == 0
   // #run: mod_int(3, 2) == 1
   // #run: mod_int(100, 7) == 2
   // #run: mod_int(15, 4) == 3
   ```

2. **ivec2, ivec3, ivec4**: Component-wise modulo
   - Multiple `// #run:` tests for each

**Note**:

- Modulo is **not defined** for float types (compile error expected)
- Undefined behavior for division by zero (not tested)
- Undefined behavior for negative operands (per spec, lines 852-853) - test only non-negative cases

## Matrix Operations Details

### Matrix Addition/Subtraction/Division

These operations are **component-wise** (not linear algebraic):

```glsl
mat2 add_mat2(mat2 a, mat2 b) {
    return a + b;  // Component-wise, not matrix multiplication
}
```

For `mat2` with column-major order:

- `a[0]` = first column (vec2)
- `a[1]` = second column (vec2)
- `a[0].x` = element at row 0, column 0
- `a[0].y` = element at row 1, column 0

### Matrix Multiplication

Matrix multiplication (`*`) is **linear algebraic** (not component-wise):

```glsl
mat2 mul_mat2(mat2 a, mat2 b) {
    return a * b;  // Linear algebraic multiply
}
```

Formula: `result[i][j] = sum(a[i][k] * b[k][j])` for all k

Example:

```
[[1, 2],     [[5, 6],     [[1*5+2*7, 1*6+2*8],     [[19, 22],
 [3, 4]]  *   [7, 8]]  =   [3*5+4*7, 3*6+4*8]]  =   [43, 50]]
```

### Vector × Matrix

- **Row vector × Matrix**: `vec2 * mat2` → `vec2`

  - Vector treated as row vector
  - Result: `[v.x*m[0].x + v.y*m[0].y, v.x*m[1].x + v.y*m[1].y]`

- **Matrix × Column vector**: `mat2 * vec2` → `vec2`
  - Vector treated as column vector
  - Result: `[m[0].x*v.x + m[1].x*v.y, m[0].y*v.x + m[1].y*v.y]`

## Test Execution Strategy

### Phase 1: Update Parsing Infrastructure

1. Update `parse_glsl_value()` in `test_run.rs` to support vector and matrix constructors
2. Update `execute_main()` to handle vector and matrix return types
3. Update `compare_results()` to use `GlslValue::approx_eq()` for vector/matrix comparisons
4. Test parsing with simple vector/matrix literals

### Phase 2: Scalar Operations

1. Create `float-add.glsl` with scalar float addition and multiple `// #run:` tests
2. Create `int-add.glsl` with scalar int addition and multiple `// #run:` tests
3. Run with bless mode to generate expectations
4. Verify tests pass

### Phase 3: Vector Operations

1. Add vector functions to `float-add.glsl` and `int-add.glsl`
2. Test vec2, vec3, vec4 (and ivec variants) with multiple `// #run:` tests each
3. Test scalar + vector operations
4. Run with bless mode
5. Verify tests pass

### Phase 4: Matrix Operations

1. Add matrix functions to `float-add.glsl`
2. Test mat2, mat3, mat4 addition (component-wise) with multiple `// #run:` tests
3. Test matrix multiplication in `float-multiply.glsl`
4. Test vector × matrix and matrix × vector
5. Run with bless mode
6. Verify tests pass

### Phase 5: Other Operations

1. Create `float-subtract.glsl`, `int-subtract.glsl`
2. Create `float-multiply.glsl`, `int-multiply.glsl`
3. Create `float-divide.glsl`, `int-divide.glsl`
4. Create `int-modulo.glsl`
5. Follow same pattern: scalar → vector → matrix, with multiple `// #run:` tests per function

## Test File Template

```glsl
// test compile
// test transform.fixed32
// test run
// target riscv32.fixed32

// ============================================================================
// Scalar Operations
// ============================================================================

float add_float(float a, float b) {
    return a + b;
}

// #run: add_float(0.0, 0.0) ~= 0.0
// #run: add_float(1.5, 2.5) ~= 4.0
// #run: add_float(-1.5, 2.5) ~= 1.0
// #run: add_float(1.5, -2.5) ~= -1.0
// #run: add_float(-1.5, -2.5) ~= -4.0
// #run: add_float(100.0, 200.0) ~= 300.0
// #run: add_float(0.001, 0.002) ~= 0.003

// ============================================================================
// Vector Operations
// ============================================================================

vec2 add_vec2(vec2 a, vec2 b) {
    return a + b;
}

// #run: add_vec2(vec2(0.0, 0.0), vec2(0.0, 0.0)) ~= vec2(0.0, 0.0)
// #run: add_vec2(vec2(1.0, 2.0), vec2(3.0, 4.0)) ~= vec2(4.0, 6.0)
// #run: add_vec2(vec2(-1.0, 2.0), vec2(3.0, -4.0)) ~= vec2(2.0, -2.0)
// #run: add_vec2(vec2(100.0, 200.0), vec2(50.0, 75.0)) ~= vec2(150.0, 275.0)

vec3 add_vec3(vec3 a, vec3 b) {
    return a + b;
}

// #run: add_vec3(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)) ~= vec3(0.0, 0.0, 0.0)
// #run: add_vec3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0)) ~= vec3(5.0, 7.0, 9.0)
// #run: add_vec3(vec3(-1.0, 2.0, -3.0), vec3(4.0, -5.0, 6.0)) ~= vec3(3.0, -3.0, 3.0)

vec4 add_vec4(vec4 a, vec4 b) {
    return a + b;
}

// #run: add_vec4(vec4(0.0, 0.0, 0.0, 0.0), vec4(0.0, 0.0, 0.0, 0.0)) ~= vec4(0.0, 0.0, 0.0, 0.0)
// #run: add_vec4(vec4(1.0, 2.0, 3.0, 4.0), vec4(5.0, 6.0, 7.0, 8.0)) ~= vec4(6.0, 8.0, 10.0, 12.0)
// #run: add_vec4(vec4(-1.0, 2.0, -3.0, 4.0), vec4(5.0, -6.0, 7.0, -8.0)) ~= vec4(4.0, -4.0, 4.0, -4.0)

// ============================================================================
// Scalar + Vector Operations
// ============================================================================

vec2 add_scalar_vec2(float s, vec2 v) {
    return s + v;
}

// #run: add_scalar_vec2(0.0, vec2(0.0, 0.0)) ~= vec2(0.0, 0.0)
// #run: add_scalar_vec2(2.0, vec2(1.0, 3.0)) ~= vec2(3.0, 5.0)
// #run: add_scalar_vec2(-1.0, vec2(2.0, 4.0)) ~= vec2(1.0, 3.0)
// #run: add_scalar_vec2(100.0, vec2(-50.0, 50.0)) ~= vec2(50.0, 150.0)

// ============================================================================
// Matrix Operations
// ============================================================================

mat2 add_mat2(mat2 a, mat2 b) {
    return a + b;
}

// #run: add_mat2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), mat2(vec2(5.0, 6.0), vec2(7.0, 8.0))) ~= mat2(vec2(6.0, 8.0), vec2(10.0, 12.0))
// #run: add_mat2(mat2(vec2(0.0, 0.0), vec2(0.0, 0.0)), mat2(vec2(0.0, 0.0), vec2(0.0, 0.0))) ~= mat2(vec2(0.0, 0.0), vec2(0.0, 0.0))
// #run: add_mat2(mat2(vec2(-1.0, 2.0), vec2(3.0, -4.0)), mat2(vec2(5.0, -6.0), vec2(-7.0, 8.0))) ~= mat2(vec2(4.0, -4.0), vec2(-4.0, 4.0))

mat3 add_mat3(mat3 a, mat3 b) {
    return a + b;
}

// #run: add_mat3(mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0)), mat3(vec3(10.0, 11.0, 12.0), vec3(13.0, 14.0, 15.0), vec3(16.0, 17.0, 18.0))) ~= mat3(vec3(11.0, 13.0, 15.0), vec3(17.0, 19.0, 21.0), vec3(23.0, 25.0, 27.0))

mat4 add_mat4(mat4 a, mat4 b) {
    return a + b;
}

// #run: add_mat4(mat4(vec4(1.0, 2.0, 3.0, 4.0), vec4(5.0, 6.0, 7.0, 8.0), vec4(9.0, 10.0, 11.0, 12.0), vec4(13.0, 14.0, 15.0, 16.0)), mat4(vec4(17.0, 18.0, 19.0, 20.0), vec4(21.0, 22.0, 23.0, 24.0), vec4(25.0, 26.0, 27.0, 28.0), vec4(29.0, 30.0, 31.0, 32.0))) ~= mat4(vec4(18.0, 20.0, 22.0, 24.0), vec4(26.0, 28.0, 30.0, 32.0), vec4(34.0, 36.0, 38.0, 40.0), vec4(42.0, 44.0, 46.0, 48.0))
```

## Notes on Vector/Matrix Literals

The test infrastructure needs to support vector and matrix constructors in run directives. Based on `GlslValue::parse()`, we need to:

1. Update `parse_glsl_value()` to parse vector constructors like `vec2(1.0, 2.0)` and matrix constructors like `mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))`
2. Use GLSL parser to extract constructor expressions
3. Handle column-major order for matrices

The format for run directives will be:

```glsl
// #run: add_vec2(vec2(1.0, 2.0), vec2(3.0, 4.0)) ~= vec2(4.0, 6.0)
// #run: add_mat2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), mat2(vec2(5.0, 6.0), vec2(7.0, 8.0))) ~= mat2(vec2(6.0, 8.0), vec2(10.0, 12.0))
```

## Success Criteria

- [ ] Vector/matrix parsing implemented in `test_run.rs`
- [ ] All 9 test files created in `filetests/math/` with type-first naming
- [ ] Each file tests scalar, vector, and matrix operations (where applicable)
- [ ] Each function has multiple `// #run:` tests covering edge cases (zero, positive, negative, mixed signs, large/small values)
- [ ] All tests pass with `cargo test -p lp-glsl-filetests --test filetests`
- [ ] CLIF expectations generated via bless mode (`// #compile:` and `// #transform:`)
- [ ] Tests verify both compilation (CLIF) and execution (run)
- [ ] Matrix multiplication correctly implements linear algebraic multiply
- [ ] Vector × matrix and matrix × vector operations tested
- [ ] Modulo tests only for integer types (no float modulo)

## Future Enhancements

- Add tests for `uint` and `uvec2/3/4` types
- Add tests for mixed-type operations (if supported)
- Add tests for operator precedence
- Add tests for compound assignment operators (`+=`, `-=`, etc.)
- Add tests for unary operators (`+`, `-`, `++`, `--`)
