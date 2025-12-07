# GLSL Compiler - Remaining Work Strategy

## Development Pace

**Note:** Phases 1-3 (all current features) were implemented in ~1 day. Timeline estimates below reflect this rapid development velocity.

## Current State

**Completed (Phase 1-4):**

- ✅ Core architecture and pipeline
- ✅ Basic types: int, bool, float
- ✅ Vectors: vec2/3/4, ivec2/3/4, bvec2/3/4
- ✅ Control flow: if/else, for, while, break, continue
- ✅ User-defined functions with parameters
- ✅ Type system with inference and validation
- ✅ Vector operations: component access, swizzling, construction
- ✅ Basic built-in functions: dot, cross, length, normalize, distance, min, max, clamp, abs, sqrt, floor, ceil, mix, step, smoothstep, fract, mod, sign
- ✅ **Matrices (mat2, mat3, mat4)** - fully implemented with:
  - Matrix construction (identity, columns, scalars)
  - Matrix indexing (column access)
  - Matrix arithmetic (+, -, × with scalar/vector/matrix)
  - Matrix built-ins: matrixCompMult, outerProduct, transpose, determinant (mat2/mat3), inverse (mat2 only)
- ✅ **Fixed-point transformation** - fully implemented (16.16 and 32.32 formats)

**Remaining Work:**

- Trigonometric functions (sin, cos, tan, asin, acos, atan, etc.)
- Exponential/logarithmic functions (exp, log, exp2, log2, inversesqrt; pow registered but not implemented)
- Structs (types exist in Type enum, but no codegen/semantic support)
- Arrays (types exist in Type enum, but no codegen/semantic support)
- Texture sampling (runtime integration)
- Additional GLSL features (uniforms, I/O qualifiers)

---

## Phase 4: Matrix Types and Operations ✅ COMPLETE

**Priority:** High  
**Status:** ✅ **COMPLETED**  
**Spec Reference:** `/Users/yona/dev/photomancer/glsl-spec/chapters/`

- `variables.adoc` lines 72-97: Matrix type definitions
- `operators.adoc` lines 1019-1098: Matrix operations
- `builtinfunctions.adoc` lines 1538-1687: Matrix functions

### Requirements

**Matrix Types (variables.adoc:72-97):**

- mat2, mat3, mat4 (2×2, 3×3, 4×4 float matrices)
- mat2x2, mat2x3, mat2x4, mat3x2, mat3x3, mat3x4, mat4x2, mat4x3, mat4x4 (non-square)
- Column-major storage (GLSL convention)
- Memory layout: consecutive columns

**Matrix Operations (operators.adoc:1019-1098):**

- Matrix construction from scalars, vectors, other matrices
- Matrix indexing: mat[col] returns column vector
- Matrix × scalar: component-wise multiplication
- Scalar × matrix: component-wise multiplication
- Matrix × vector: linear algebra multiplication
- Vector × matrix: linear algebra multiplication
- Matrix × matrix: linear algebra multiplication
- Component-wise operations: mat + mat, mat - mat

**Matrix Built-ins (builtinfunctions.adoc:1538-1687):**

- `matrixCompMult(mat, mat)` - component-wise multiply
- `outerProduct(vec, vec)` - outer product → matrix
- `transpose(mat)` - matrix transpose
- `determinant(mat)` - matrix determinant
- `inverse(mat)` - matrix inverse

### Implementation Strategy

1. **Type System Updates** (`semantic/types.rs`)

   - Add matrix types to Type enum
   - Implement `is_matrix()`, `matrix_dims()` helpers
   - Matrix → Cranelift type mapping (struct or array)

2. **Storage Representation** (`codegen/context.rs`)

   - Column-major array layout: mat3 = [vec3; 3] = 9 floats
   - Stack allocation for matrices (via VirtualValue)
   - Or register struct for mat2 (4 floats)

3. **Construction** (`codegen/expr.rs`)

   - From scalars: `mat3(1.0)` → identity matrix
   - From columns: `mat3(vec1, vec2, vec3)`
   - From mixed: `mat2(float, float, float, float)`

4. **Indexing** (`codegen/expr.rs`)

   - `mat[i]` → extract column vector (offset = i \* col_size)
   - `mat[i][j]` → extract scalar element

5. **Arithmetic Operations**

   - Matrix × scalar: expand to component-wise fmul
   - Matrix × vector: dot products of rows with vector
   - Matrix × matrix: standard linear algebra multiplication
   - Component-wise: iterate over all elements

6. **Built-in Functions** (`codegen/builtins.rs`)
   - Implement each as separate function
   - Use BLAS-like patterns for matrix multiply
   - Transpose: swap rows/columns

### Tests Required

**Matrix Construction Tests (variables.adoc:72-97):**

```glsl
// Test: mat3_construct_identity.glsl
// Spec: variables.adoc:72-97 - Matrix type definitions
mat3 main() {
    return mat3(1.0); // Identity matrix
}
// CHECK: 1.0 on diagonal, 0.0 elsewhere

// Test: mat3_construct_columns.glsl
// Spec: variables.adoc:72-97
mat3 main() {
    vec3 col0 = vec3(1.0, 2.0, 3.0);
    vec3 col1 = vec3(4.0, 5.0, 6.0);
    vec3 col2 = vec3(7.0, 8.0, 9.0);
    return mat3(col0, col1, col2);
}
// CHECK: column-major storage

// Test: mat2_construct_scalars.glsl
// Spec: variables.adoc:72-97
mat2 main() {
    return mat2(1.0, 2.0, 3.0, 4.0);
}
```

**Matrix Operations Tests (operators.adoc:1019-1098):**

```glsl
// Test: mat3_index_column.glsl
// Spec: operators.adoc:1019-1098 - Matrix indexing
vec3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return m[1]; // Second column
}
// run: == vec3(4.0, 5.0, 6.0)

// Test: mat_mul_scalar.glsl
// Spec: operators.adoc:1019-1098 - Matrix arithmetic
mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return m * 2.0;
}
// run: == mat2(2.0, 4.0, 6.0, 8.0)

// Test: mat_mul_vec.glsl
// Spec: operators.adoc:1019-1098 - Matrix-vector multiply
vec3 main() {
    mat3 m = mat3(1.0);  // Identity
    vec3 v = vec3(2.0, 3.0, 4.0);
    return m * v;
}
// run: == vec3(2.0, 3.0, 4.0)

// Test: mat_mul_mat.glsl
// Spec: operators.adoc:1019-1098 - Matrix-matrix multiply
mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(5.0, 6.0, 7.0, 8.0);
    return a * b;
}
// CHECK: standard matrix multiply result
```

**Matrix Built-ins Tests (builtinfunctions.adoc:1538-1687):**

```glsl
// Test: matrixCompMult.glsl
// Spec: builtinfunctions.adoc:1538-1687
mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}
// run: == mat2(2.0, 4.0, 6.0, 8.0)

// Test: transpose.glsl
// Spec: builtinfunctions.adoc:1538-1687
mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose(m);
}
// CHECK: rows become columns

// Test: determinant.glsl
// Spec: builtinfunctions.adoc:1538-1687
float main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return determinant(m);
}
// run: == -2.0  (1*4 - 2*3)
```

### Success Criteria

- [x] All matrix types defined and usable
- [x] Matrix construction works (identity, columns, scalars)
- [x] Matrix indexing returns correct columns/elements
- [x] Matrix × scalar, vector, matrix operations correct
- [x] All 5 matrix built-ins implemented (matrixCompMult, outerProduct, transpose, determinant, inverse)
- [x] Comprehensive test suite in `crates/lp-glsl-filetests/filetests/matrices/`

**Note:** Determinant supports mat2 and mat3 (mat4 not yet implemented). Inverse supports mat2 only (mat3/mat4 not yet implemented).

---

## Phase 5: Trigonometric Functions

**Priority:** High (needed for shader effects)  
**Status:** ❌ **NOT STARTED**  
**Estimated Effort:** 2-3 hours  
**Spec Reference:** `builtinfunctions.adoc` lines 122-310

### Requirements

**Angle and Trigonometry Functions (builtinfunctions.adoc:122-310):**

- `radians(degrees)` - degrees to radians
- `degrees(radians)` - radians to degrees
- `sin(angle)` - sine
- `cos(angle)` - cosine
- `tan(angle)` - tangent
- `asin(x)` - arc sine
- `acos(x)` - arc cosine
- `atan(y, x)` - arc tangent (2-arg)
- `atan(y_over_x)` - arc tangent (1-arg)
- `sinh(x)` - hyperbolic sine
- `cosh(x)` - hyperbolic cosine
- `tanh(x)` - hyperbolic tangent
- `asinh(x)` - hyperbolic arc sine
- `acosh(x)` - hyperbolic arc cosine
- `atanh(x)` - hyperbolic arc tangent

All operate component-wise on vectors.

### Implementation Strategy

1. **For Native Targets** (with libm)

   - Generate Cranelift libcalls to `sinf`, `cosf`, `tanf`, etc.
   - Use `builder.ins().call()` with external function declarations
   - Same pattern as existing `sqrt` implementation

2. **For Fixed-Point/RISC-V**

   - CORDIC algorithm implementations
   - Lookup tables for common angles
   - Taylor series approximations
   - Store in fixed-point runtime library

3. **Builtin Registry** (`semantic/builtins.rs`)

   - Register each function with signatures
   - Support scalar and vector variants (genFType)

4. **Code Generation** (`codegen/builtins.rs`)
   - Scalar: direct libcall
   - Vector: component-wise expansion

### Tests Required

**Basic Trig Tests (builtinfunctions.adoc:122-310):**

```glsl
// Test: sin_scalar.glsl
// Spec: builtinfunctions.adoc:149-156 - sin function
float main() {
    return sin(0.0);
}
// run: == 0.0

// Test: cos_scalar.glsl
// Spec: builtinfunctions.adoc:157-164 - cos function
float main() {
    return cos(0.0);
}
// run: == 1.0

// Test: tan_scalar.glsl
// Spec: builtinfunctions.adoc:165-172 - tan function
float main() {
    float pi_4 = 0.785398163;  // π/4
    return tan(pi_4);
}
// run: ≈ 1.0 (within tolerance)

// Test: radians_degrees.glsl
// Spec: builtinfunctions.adoc:133-147 - angle conversion
float main() {
    float deg = 180.0;
    float rad = radians(deg);
    return degrees(rad);
}
// run: == 180.0

// Test: sin_vec3.glsl
// Spec: builtinfunctions.adoc:149-156 - component-wise
vec3 main() {
    vec3 angles = vec3(0.0, 0.0, 0.0);
    return sin(angles);
}
// run: == vec3(0.0, 0.0, 0.0)

// Test: atan2.glsl
// Spec: builtinfunctions.adoc:196-211 - atan with 2 args
float main() {
    return atan(1.0, 1.0);
}
// run: ≈ 0.785398 (π/4)
```

**Arc Functions Tests (builtinfunctions.adoc:173-228):**

```glsl
// Test: asin_scalar.glsl
// Spec: builtinfunctions.adoc:173-183
float main() {
    return asin(0.5);
}
// run: ≈ 0.523599 (π/6)

// Test: acos_scalar.glsl
// Spec: builtinfunctions.adoc:184-195
float main() {
    return acos(0.5);
}
// run: ≈ 1.047198 (π/3)
```

**Hyperbolic Tests (builtinfunctions.adoc:229-310):**

```glsl
// Test: sinh_scalar.glsl
// Spec: builtinfunctions.adoc:229-237
float main() {
    return sinh(0.0);
}
// run: == 0.0

// Test: cosh_scalar.glsl
// Spec: builtinfunctions.adoc:238-246
float main() {
    return cosh(0.0);
}
// run: == 1.0
```

### Success Criteria

- [ ] All 15 trig functions implemented
- [ ] Scalar versions work with libcalls
- [ ] Vector versions expand component-wise
- [ ] Minimum 10 tests pass with correct values
- [ ] Fixed-point transformation already available (16.16 and 32.32 formats)

---

## Phase 6: Exponential and Logarithmic Functions

**Priority:** Medium-High  
**Status:** ❌ **PARTIALLY STARTED** (pow registered but not implemented)  
**Estimated Effort:** 1-2 hours  
**Spec Reference:** `builtinfunctions.adoc` lines 311-409

**Current State:**

- `pow()` is registered in builtin signatures but returns error "needs exp/log"
- `sqrt()` is fully implemented
- `exp()`, `log()`, `exp2()`, `log2()`, `inversesqrt()` not yet implemented

### Requirements

**Exponential Functions (builtinfunctions.adoc:311-409):**

- `pow(x, y)` - x raised to y power (registered but returns error - needs exp/log)
- `exp(x)` - natural exponentiation (e^x) - **NOT IMPLEMENTED**
- `log(x)` - natural logarithm - **NOT IMPLEMENTED**
- `exp2(x)` - 2^x - **NOT IMPLEMENTED**
- `log2(x)` - base 2 logarithm - **NOT IMPLEMENTED**
- `sqrt(x)` - square root - ✅ **IMPLEMENTED**
- `inversesqrt(x)` - 1/sqrt(x) - **NOT IMPLEMENTED**

All operate component-wise on vectors.

### Implementation Strategy

Same as trigonometric functions:

- Native: libcalls to `powf`, `expf`, `logf`, etc.
- Fixed-point: approximation algorithms
- Component-wise for vectors

### Tests Required

```glsl
// Test: pow_scalar.glsl
// Spec: builtinfunctions.adoc:319-328
float main() {
    return pow(2.0, 3.0);
}
// run: == 8.0

// Test: exp_scalar.glsl
// Spec: builtinfunctions.adoc:329-337
float main() {
    return exp(0.0);
}
// run: == 1.0

// Test: log_scalar.glsl
// Spec: builtinfunctions.adoc:338-346
float main() {
    return log(2.718282);  // e
}
// run: ≈ 1.0

// Test: exp2_scalar.glsl
// Spec: builtinfunctions.adoc:347-355
float main() {
    return exp2(3.0);
}
// run: == 8.0

// Test: log2_scalar.glsl
// Spec: builtinfunctions.adoc:356-364
float main() {
    return log2(8.0);
}
// run: == 3.0

// Test: inversesqrt_scalar.glsl
// Spec: builtinfunctions.adoc:373-381
float main() {
    return inversesqrt(4.0);
}
// run: == 0.5
```

### Success Criteria

- [ ] All 7 exponential/log functions implemented (currently only sqrt works)
- [ ] pow() implemented via exp/log or libcall
- [ ] Minimum 6 tests pass with correct values
- [ ] Component-wise vector support

---

## Phase 7: Struct Types

**Priority:** Medium  
**Status:** ❌ **NOT STARTED** (types exist but no implementation)  
**Estimated Effort:** 4-6 hours  
**Spec Reference:** `variables.adoc` lines 564-719

**Current State:**

- `Type::Struct(StructId)` exists in type system
- No semantic analysis for struct declarations
- No codegen for struct construction, member access, or storage

### Requirements

**Struct Definitions (variables.adoc:564-719):**

- User-defined struct types
- Member declarations with types
- Nested structs
- Struct constructors
- Member access (dot notation)
- Struct initialization

**Example from spec:**

```glsl
struct Light {
    vec3 position;
    vec3 color;
    float intensity;
};

Light myLight = Light(vec3(1.0, 2.0, 3.0), vec3(1.0, 0.5, 0.3), 0.8);
vec3 pos = myLight.position;
```

### Implementation Strategy

1. **Type System** (`semantic/types.rs`)

   - StructDef with field list, offsets, alignment
   - Register structs in symbol table
   - Calculate memory layout (std140/std430)

2. **Semantic Analysis** (`semantic/mod.rs`)

   - Parse struct declarations
   - Build field map with offsets
   - Validate member access

3. **Code Generation** (`codegen/`)

   - Stack allocation for struct instances
   - Member access via offset calculation
   - Constructor expansion to field assignments
   - Reference: cranelift-examples `lowering-structs/`

4. **Memory Layout**
   - Follow GLSL alignment rules
   - Padding between fields
   - Whole struct alignment

### Tests Required

```glsl
// Test: struct_definition.glsl
// Spec: variables.adoc:564-719 - Struct declarations
struct Point {
    float x;
    float y;
};

float main() {
    Point p = Point(3.0, 4.0);
    return p.x;
}
// run: == 3.0

// Test: struct_member_access.glsl
// Spec: variables.adoc:564-719 - Member access
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

// Test: struct_member_assign.glsl
// Spec: variables.adoc:564-719 - Member modification
struct Data {
    int value;
};

int main() {
    Data d = Data(10);
    d.value = 20;
    return d.value;
}
// run: == 20

// Test: struct_nested.glsl
// Spec: variables.adoc:564-719 - Nested structs
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

// Test: struct_with_vector.glsl
// Spec: variables.adoc:564-719 - Structs with vectors
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

### Success Criteria

- [ ] Struct declarations parsed and validated
- [ ] Struct construction works
- [ ] Member access generates correct offsets
- [ ] Member assignment works
- [ ] Nested structs supported
- [ ] Structs with vectors/matrices work
- [ ] Minimum 8 tests pass

---

## Phase 8: Array Types

**Priority:** Medium  
**Status:** ❌ **NOT STARTED** (types exist but no implementation)  
**Estimated Effort:** 3-4 hours  
**Spec Reference:** `variables.adoc` lines 359-515

**Current State:**

- `Type::Array(Box<Type>, usize)` exists in type system
- No semantic analysis for array declarations
- No codegen for array construction, indexing, or storage
- Note: Matrix indexing uses `Expr::Bracket` but only for matrices, not general arrays

### Requirements

**Array Declarations (variables.adoc:359-515):**

- Fixed-size arrays: `float values[4];`
- Array initialization: `float values[3] = float[3](1.0, 2.0, 3.0);`
- Array indexing: `values[i]`
- Multidimensional arrays: `float matrix[3][3];`
- Arrays as function parameters
- Arrays in structs

**Restrictions:**

- Array size must be compile-time constant
- No dynamic arrays

### Implementation Strategy

1. **Type System**

   - `Type::Array(Box<Type>, usize)`
   - Size validation at compile time
   - Multi-dimensional = nested arrays

2. **Storage**

   - Stack allocation for local arrays
   - Consecutive memory layout
   - Calculate strides for multi-dimensional

3. **Indexing**
   - Bounds checking (optional, for safety)
   - Offset = index × element_size
   - Multi-dimensional: flatten to 1D

### Tests Required

```glsl
// Test: array_declaration.glsl
// Spec: variables.adoc:359-515 - Array declarations
float main() {
    float values[3] = float[3](1.0, 2.0, 3.0);
    return values[1];
}
// run: == 2.0

// Test: array_indexing.glsl
// Spec: variables.adoc:359-515 - Array indexing
int main() {
    int data[5] = int[5](10, 20, 30, 40, 50);
    return data[3];
}
// run: == 40

// Test: array_assign.glsl
// Spec: variables.adoc:359-515 - Array element assignment
float main() {
    float vals[2];
    vals[0] = 5.0;
    vals[1] = 10.0;
    return vals[0] + vals[1];
}
// run: == 15.0

// Test: array_vec3.glsl
// Spec: variables.adoc:359-515 - Arrays of vectors
vec3 main() {
    vec3 positions[2] = vec3[2](vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
    return positions[1];
}
// run: == vec3(4.0, 5.0, 6.0)

// Test: array_in_struct.glsl
// Spec: variables.adoc:359-515 - Arrays as struct members
struct Batch {
    float values[3];
};

float main() {
    Batch b = Batch(float[3](1.0, 2.0, 3.0));
    return b.values[2];
}
// run: == 3.0
```

### Success Criteria

- [ ] Fixed-size array declarations work
- [ ] Array initialization with constructor
- [ ] Array indexing generates correct offsets
- [ ] Arrays of scalars, vectors, structs
- [ ] Multi-dimensional arrays (bonus)
- [ ] Minimum 6 tests pass

---

## Phase 9: Additional Built-in Functions

**Priority:** Low-Medium  
**Estimated Effort:** 2-3 hours  
**Spec Reference:** `builtinfunctions.adoc` various sections

### Requirements

**Common Functions (builtinfunctions.adoc:410-771):**
Already implemented: abs, sign, floor, ceil, fract, mod, min, max, clamp, mix, step, smoothstep

Still needed:

- `trunc(x)` - truncate to integer
- `round(x)` - round to nearest integer
- `roundEven(x)` - round to nearest even integer
- `modf(x, out i)` - separate integer and fractional parts
- `isnan(x)` - test for NaN
- `isinf(x)` - test for infinity

**Geometric Functions (builtinfunctions.adoc:772-908):**
Already implemented: length, distance, dot, cross, normalize

Still needed:

- `faceforward(N, I, Nref)` - orient normal
- `reflect(I, N)` - reflection direction
- `refract(I, N, eta)` - refraction direction

**Bitwise Functions (builtinfunctions.adoc:1688-2090):**

- Left for later (less critical for shaders)

### Tests Required

```glsl
// Test: trunc_scalar.glsl
// Spec: builtinfunctions.adoc:410-771
float main() {
    return trunc(3.7);
}
// run: == 3.0

// Test: round_scalar.glsl
// Spec: builtinfunctions.adoc:410-771
float main() {
    return round(3.5);
}
// run: == 4.0

// Test: reflect_vec3.glsl
// Spec: builtinfunctions.adoc:772-908 - Geometric functions
vec3 main() {
    vec3 I = vec3(1.0, -1.0, 0.0);
    vec3 N = vec3(0.0, 1.0, 0.0);
    return reflect(I, N);
}
// run: == vec3(1.0, 1.0, 0.0)
```

### Success Criteria

- [ ] 6 additional common functions implemented
- [ ] 3 geometric functions implemented
- [ ] Minimum 5 tests pass

---

## Phase 10: Texture Sampling (Runtime Integration)

**Priority:** Low (requires runtime)  
**Estimated Effort:** 2-3 hours  
**Spec Reference:** `builtinfunctions.adoc` lines 2091-2800

### Requirements

**Texture Access Functions (builtinfunctions.adoc:2091-2800):**

- `texture(sampler2D, vec2)` - basic texture lookup
- `texture(samplerCube, vec3)` - cubemap lookup
- `textureSize(sampler, lod)` - get texture dimensions
- `texelFetch(sampler, ivec2, lod)` - fetch specific texel

**Sampler Types:**

- sampler2D, sampler3D, samplerCube
- isampler2D, usampler2D (integer textures)

### Implementation Strategy

1. **Type System**

   - Add sampler types (opaque handles)
   - Validate sampler usage (uniforms only)

2. **Code Generation**

   - Generate external function calls
   - Pass sampler handle + coordinates
   - Runtime provides implementation

3. **Runtime Interface**
   - Define C ABI for texture functions
   - Link with shader runtime library
   - Actual sampling done by runtime (not compiler)

### Tests Required

```glsl
// Test: texture2D_basic.glsl
// Spec: builtinfunctions.adoc:2091-2800 - Texture access
uniform sampler2D tex;

vec4 main() {
    vec2 uv = vec2(0.5, 0.5);
    return texture(tex, uv);
}
// CHECK: call to __glsl_texture2D
// runtime test: requires texture data

// Test: textureSize.glsl
// Spec: builtinfunctions.adoc:2091-2800
uniform sampler2D tex;

ivec2 main() {
    return textureSize(tex, 0);
}
// CHECK: call to __glsl_textureSize
```

### Success Criteria

- [ ] Sampler types recognized
- [ ] Texture functions generate external calls
- [ ] ABI defined for runtime integration
- [ ] Minimum 2 tests compile (runtime execution separate)

---

## Phase 11: Shader I/O and Uniforms

**Priority:** Medium (for complete shaders)  
**Estimated Effort:** 3-4 hours  
**Spec Reference:** `variables.adoc` lines 1230-1550

### Requirements

**Storage Qualifiers (variables.adoc:1230-1550):**

- `uniform` - read-only values from application
- `in` - input from previous shader stage
- `out` - output to next shader stage
- `const` - compile-time constants

**Built-in Variables:**

- `gl_Position` (vertex shader output)
- `gl_FragColor` / `gl_FragData` (fragment shader output)
- `gl_FragCoord` (fragment shader input)

### Implementation Strategy

1. **Semantic Analysis**

   - Track storage qualifiers in symbol table
   - Validate usage (uniforms are read-only, etc.)

2. **Code Generation**

   - Uniforms: pass as function parameters
   - Inputs: pass as function parameters
   - Outputs: return from function or out parameters
   - Built-ins: special handling

3. **ABI Definition**
   - Define calling convention for shader entry point
   - Uniform block layouts (std140/std430)
   - Input/output attribute locations

### Tests Required

```glsl
// Test: uniform_declaration.glsl
// Spec: variables.adoc:1230-1550 - Storage qualifiers
uniform float time;

float main() {
    return time;
}
// CHECK: time passed as parameter

// Test: in_out_variables.glsl
// Spec: variables.adoc:1230-1550
in vec3 position;
out vec4 fragColor;

vec4 main() {
    fragColor = vec4(position, 1.0);
    return fragColor;
}
// CHECK: input parameter, output written
```

### Success Criteria

- [ ] Uniform qualifier recognized and validated
- [ ] In/out qualifiers work
- [ ] Storage qualifiers prevent invalid operations
- [ ] Minimum 3 tests pass

---

## Phase 12: Optimizations

**Priority:** Low (performance)  
**Estimated Effort:** Ongoing

### Optimization Opportunities

1. **Constant Folding**

   - Evaluate constant expressions at compile time
   - `2.0 * 3.0` → `6.0`

2. **Dead Code Elimination**

   - Remove unused variables and functions
   - Remove unreachable code after return

3. **Common Subexpression Elimination**

   - Reuse computed values
   - Important for expensive operations (sqrt, normalize)

4. **Loop Unrolling**

   - Unroll small constant-count loops
   - Helps with fixed-point performance

5. **Vector Operation Fusion**
   - Combine multiple vector ops
   - Use SIMD when available

### Implementation

- Add optimization passes before codegen
- Use Cranelift's optimization framework
- Add flags to enable/disable optimizations

---

## Testing Strategy

### Test Organization

```
crates/lp-glsl-filetests/filetests/
├── matrices/           (Phase 4)
│   ├── mat3_construct_identity.glsl
│   ├── mat_mul_vec.glsl
│   └── transpose.glsl
├── trig/               (Phase 5)
│   ├── sin_scalar.glsl
│   ├── cos_vec3.glsl
│   └── atan2.glsl
├── exponential/        (Phase 6)
│   ├── pow_scalar.glsl
│   ├── exp_scalar.glsl
│   └── log2_scalar.glsl
├── structs/            (Phase 7)
│   ├── struct_definition.glsl
│   ├── struct_nested.glsl
│   └── struct_with_vector.glsl
├── arrays/             (Phase 8)
│   ├── array_declaration.glsl
│   ├── array_indexing.glsl
│   └── array_vec3.glsl
├── additional_builtins/ (Phase 9)
│   ├── trunc_scalar.glsl
│   └── reflect_vec3.glsl
├── textures/           (Phase 10)
│   └── texture2D_basic.glsl
└── io/                 (Phase 11)
    └── uniform_declaration.glsl
```

### Test Format

Every test file should include:

```glsl
// Test: [description]
// Spec: [chapter.adoc]:[line-range] - [section name]
[glsl code]
// CHECK: [expected IR pattern]
// run: [expected result] or [comparison]
```

### Validation Levels

1. **Compile Test** - GLSL → CLIF IR
2. **Run Test** - Execute and check result
3. **Spec Compliance** - Verify against GLSL spec behavior

---

## Implementation Priority Order

1. ✅ **Phase 4: Matrices** (critical for 3D transforms) - **COMPLETE**
2. **Phase 5: Trigonometry** (critical for visual effects) - **NEXT PRIORITY**
3. **Phase 6: Exponential/Log** (less critical, quick to add) - **PARTIALLY STARTED**
4. **Phase 7: Structs** (architectural, enables complex data)
5. **Phase 11: Uniforms/I/O** (needed for real shaders)
6. **Phase 8: Arrays** (useful but can work around)
7. **Phase 9: Additional Built-ins** (nice to have)
8. **Phase 10: Textures** (requires runtime, defer)
9. **Phase 12: Optimizations** (performance, ongoing)

---

## Timeline Estimate

Based on current development velocity (~1 day for phases 1-3):

- ✅ **Phase 4 (Matrices):** 4-6 hours - **COMPLETE**
- **Phase 5 (Trig):** 2-3 hours - **NEXT**
- **Phase 6 (Exp/Log):** 1-2 hours - **PARTIALLY STARTED**
- **Phase 7 (Structs):** 4-6 hours
- **Phase 8 (Arrays):** 3-4 hours
- **Phase 9 (Additional Built-ins):** 2-3 hours
- **Phase 10 (Textures):** 2-3 hours
- **Phase 11 (Uniforms/I/O):** 3-4 hours

**Remaining:** ~18-25 hours (2-3 days of focused work)

---

## Success Metrics

**Per Phase:**

- All spec-referenced features implemented
- All tests passing (compile and run)
- No regressions in previous tests
- Documentation updated

**Overall:**

- Can compile real-world GLSL shaders
- Spec-compliant type system and operations
- Fixed-point transformation works for all operations
- Performance acceptable on RISC-V target
