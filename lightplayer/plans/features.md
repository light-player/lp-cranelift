# GLSL Compiler Feature Checklist

## Types

### Scalar Types

- ✅ `int` - Signed integer
- ✅ `bool` - Boolean
- ✅ `float` - Single-precision floating-point
- ❌ `uint` - Unsigned integer
- ❌ `double` - Double-precision floating-point

### Vector Types

- ✅ `vec2`, `vec3`, `vec4` - Float vectors
- ✅ `ivec2`, `ivec3`, `ivec4` - Integer vectors
- ✅ `bvec2`, `bvec3`, `bvec4` - Boolean vectors
- ❌ `uvec2`, `uvec3`, `uvec4` - Unsigned integer vectors
- ❌ `dvec2`, `dvec3`, `dvec4` - Double vectors

### Matrix Types

- ✅ `mat2`, `mat3`, `mat4` - Float matrices
- ❌ `mat2x3`, `mat2x4`, `mat3x2`, `mat3x4`, `mat4x2`, `mat4x3` - Non-square matrices
- ❌ `dmat2`, `dmat3`, `dmat4` - Double matrices
- ❌ `dmat2x3`, `dmat2x4`, `dmat3x2`, `dmat3x4`, `dmat4x2`, `dmat4x3` - Double non-square matrices

### Composite Types

- ❌ `struct` - User-defined structures
- ❌ Arrays - Fixed-size arrays
- ❌ Samplers - Texture samplers (sampler2D, etc.)

## Operators

### Unary Operators

- ✅ `+` - Unary plus
- ✅ `-` - Unary minus (negation)
- ✅ `!` - Logical NOT
- ✅ `++` - Pre-increment (on scalars, vectors, matrices, components)
- ✅ `--` - Pre-decrement (on scalars, vectors, matrices, components)
- ✅ `++` - Post-increment (on scalars, vectors, matrices, components)
- ✅ `--` - Post-decrement (on scalars, vectors, matrices, components)
- ❌ `~` - Bitwise NOT (one's complement)

### Binary Arithmetic Operators

- ✅ `+` - Addition (scalars, vectors, matrices)
- ✅ `-` - Subtraction (scalars, vectors, matrices)
- ✅ `*` - Multiplication (scalars, vectors, matrices, matrix-vector, matrix-matrix)
- ✅ `/` - Division (scalars, vectors)
- ✅ `%` - Modulus (integers, integer vectors)

### Binary Comparison Operators

- ✅ `<` - Less than (scalars only)
- ✅ `>` - Greater than (scalars only)
- ✅ `<=` - Less than or equal (scalars only)
- ✅ `>=` - Greater than or equal (scalars only)
- ✅ `==` - Equality (all types)
- ✅ `!=` - Inequality (all types)
- ❌ Vector component-wise comparisons (use built-ins: lessThan, greaterThan, etc.)

### Binary Logical Operators

- ✅ `&&` - Logical AND (scalars only)
- ⚠️ `||` - Logical OR (scalars only) - Code exists but may be incomplete
- ❌ `^^` - Logical XOR (scalars only)

### Binary Bitwise Operators

- ❌ `&` - Bitwise AND
- ❌ `|` - Bitwise OR
- ❌ `^` - Bitwise XOR
- ❌ `<<` - Left shift
- ❌ `>>` - Right shift

### Assignment Operators

- ✅ `=` - Simple assignment
- ❌ `+=` - Add and assign
- ❌ `-=` - Subtract and assign
- ❌ `*=` - Multiply and assign
- ❌ `/=` - Divide and assign
- ❌ `%=` - Modulus and assign
- ❌ `<<=` - Left shift and assign
- ❌ `>>=` - Right shift and assign
- ❌ `&=` - Bitwise AND and assign
- ❌ `|=` - Bitwise OR and assign
- ❌ `^=` - Bitwise XOR and assign

### Other Operators

- ✅ `? :` - Ternary conditional operator
- ✅ `.` - Component access and swizzling
- ✅ `[]` - Array/matrix subscripting (vectors, matrices)
- ✅ `()` - Function calls and constructors
- ❌ `,` - Sequence operator

## Statements

### Control Flow

- ✅ `if` - Conditional statement
- ✅ `if-else` - Conditional with else branch
- ✅ `for` - For loops (with init, condition, update)
- ✅ `while` - While loops
- ✅ `do-while` - Do-while loops
- ❌ `switch` - Switch statements
- ❌ `case` - Case labels
- ❌ `default` - Default label

### Jump Statements

- ✅ `return` - Return with value
- ✅ `return` - Return without value (void)
- ✅ `break` - Break from loops
- ✅ `continue` - Continue to next iteration
- ❌ `discard` - Fragment shader discard

### Declarations

- ✅ Variable declarations with initialization
- ✅ Variable declarations without initialization
- ✅ Multiple variables in one declaration
- ❌ Struct declarations
- ❌ Array declarations
- ❌ Function prototypes/declarations (only definitions supported)

## Built-in Functions

### Angle and Trigonometry

- ✅ `radians()` - Convert degrees to radians
- ✅ `degrees()` - Convert radians to degrees
- ✅ `sin()` - Sine
- ✅ `cos()` - Cosine
- ✅ `tan()` - Tangent
- ✅ `asin()` - Arc sine
- ✅ `acos()` - Arc cosine
- ✅ `atan()` - Arc tangent (single arg)
- ✅ `atan()` - Arc tangent (two args)
- ✅ `sinh()` - Hyperbolic sine
- ✅ `cosh()` - Hyperbolic cosine
- ✅ `tanh()` - Hyperbolic tangent
- ✅ `asinh()` - Inverse hyperbolic sine
- ✅ `acosh()` - Inverse hyperbolic cosine
- ✅ `atanh()` - Inverse hyperbolic tangent

### Exponential Functions

- ✅ `pow()` - Power function
- ❌ `exp()` - Natural exponentiation
- ❌ `log()` - Natural logarithm
- ❌ `exp2()` - Base 2 exponentiation
- ❌ `log2()` - Base 2 logarithm
- ✅ `sqrt()` - Square root
- ❌ `inversesqrt()` - Inverse square root

### Common Functions

- ✅ `abs()` - Absolute value
- ✅ `sign()` - Sign function
- ✅ `floor()` - Floor function
- ❌ `trunc()` - Truncate
- ❌ `round()` - Round
- ❌ `roundEven()` - Round to nearest even
- ✅ `ceil()` - Ceiling function
- ✅ `fract()` - Fractional part
- ✅ `mod()` - Modulus
- ✅ `min()` - Minimum (scalars and vectors)
- ✅ `max()` - Maximum (scalars and vectors)
- ✅ `clamp()` - Clamp (scalars and vectors)

### Geometric Functions

- ✅ `length()` - Vector length
- ✅ `distance()` - Distance between points
- ✅ `dot()` - Dot product
- ✅ `cross()` - Cross product
- ✅ `normalize()` - Normalize vector
- ❌ `faceforward()` - Face forward
- ❌ `reflect()` - Reflection
- ❌ `refract()` - Refraction

### Matrix Functions

- ✅ `matrixCompMult()` - Component-wise matrix multiplication
- ✅ `outerProduct()` - Outer product
- ✅ `transpose()` - Matrix transpose
- ✅ `determinant()` - Matrix determinant
- ✅ `inverse()` - Matrix inverse

### Interpolation Functions

- ✅ `mix()` - Linear interpolation
- ✅ `step()` - Step function
- ✅ `smoothstep()` - Smooth step function

### Vector Relational Functions

- ❌ `lessThan()` - Component-wise less than
- ❌ `lessThanEqual()` - Component-wise less than or equal
- ❌ `greaterThan()` - Component-wise greater than
- ❌ `greaterThanEqual()` - Component-wise greater than or equal
- ❌ `equal()` - Component-wise equality
- ❌ `notEqual()` - Component-wise inequality
- ❌ `any()` - Any component true
- ❌ `all()` - All components true
- ❌ `not()` - Component-wise logical NOT

### Texture Functions

- ❌ All texture sampling functions (texture, texture2D, etc.)

### Other Built-ins

- ❌ `exp()`, `log()`, `exp2()`, `log2()`, `inversesqrt()`
- ❌ `trunc()`, `round()`, `roundEven()`
- ❌ `min()`, `max()`, `clamp()` (vector versions)
- ❌ `faceforward()`, `reflect()`, `refract()`
- ❌ All vector relational functions
- ❌ All texture functions

## Expressions

### Constructors

- ✅ Scalar constructors (int(), float(), bool())
- ✅ Vector constructors (vec2(), vec3(), vec4(), etc.)
- ✅ Matrix constructors (mat2(), mat3(), mat4())
- ✅ Type conversions via constructors
- ❌ Struct constructors
- ❌ Array constructors

### Component Access

- ✅ Single component access (v.x, v.y, etc.)
- ✅ Swizzling (v.xy, v.rgb, etc.)
- ✅ Swizzle assignment (v.xy = vec2(...))
- ✅ Vector indexing (v[0], v[1], etc.)
- ✅ Matrix indexing (m[0], m[0][1], etc.)
- ❌ Array indexing (when arrays are implemented)

## Functions

### User-Defined Functions

- ✅ Function definitions
- ✅ Function calls
- ✅ Parameters (in only)
- ✅ Return values (scalars, vectors, matrices)
- ❌ `out` parameters
- ❌ `inout` parameters
- ❌ Function overloading (partial - type checking exists but may need refinement)
- ❌ Recursive functions (spec disallows, but error checking needed)

### Function Qualifiers

- ❌ `const` parameter qualifier
- ❌ Precision qualifiers on parameters/return types

## Variables

### Variable Qualifiers

- ❌ `const` - Constant variables
- ❌ `uniform` - Uniform variables
- ❌ `in` - Input variables
- ❌ `out` - Output variables
- ❌ `inout` - Input/output variables
- ❌ Precision qualifiers (`lowp`, `mediump`, `highp`)

### Variable Scope

- ✅ Local variables
- ✅ Function parameters
- ✅ Global variables (in main function context)
- ✅ Block scope

## Special Features

### Preprocessor

- ❌ `#define` - Macro definitions
- ❌ `#undef` - Undefine macros
- ❌ `#if`, `#ifdef`, `#ifndef`, `#else`, `#elif`, `#endif` - Conditional compilation
- ❌ `#error` - Error directive
- ❌ `#pragma` - Pragma directive
- ❌ `#extension` - Extension directive
- ❌ `#version` - Version directive
- ❌ `#line` - Line directive

### Other

- ❌ Built-in variables (gl_FragCoord, etc.)
- ❌ Layout qualifiers
- ❌ Memory qualifiers (`coherent`, `volatile`, `restrict`, etc.)
- ❌ Invariance qualifiers (`invariant`, `precise`)

## Known Issues / Partial Implementation

### Increment/Decrement Operators

- ✅ Matrix increment/decrement: Fully implemented - matrices are recognized as numeric types and increment/decrement works component-wise per GLSL spec (operators.adoc:858)
- ❌ Array element increment/decrement: Not yet implemented (arrays not supported in compiler)
- ✅ Pre-increment/decrement: Fully implemented with comprehensive test coverage (scalars, vectors, matrices, components)

### Assignment Operators

- ⚠️ Only simple assignment (`=`) is implemented
- ⚠️ Compound assignment operators (`+=`, `-=`, etc.) not implemented

### Built-in Functions

- ⚠️ Some functions may have incomplete vector support (e.g., min/max/clamp)
- ⚠️ Vector relational functions not implemented (use built-ins instead of operators)

### Type System

- ⚠️ `uint` type exists in type system but may not be fully supported in all operations
- ⚠️ Non-square matrices not implemented

## Test Coverage Gaps

- ✅ All increment/decrement test cases: Comprehensive test suite created covering scalars, vectors, matrices, components, for loops, and edge cases
- Compound assignment operators
- Bitwise operators
- Logical OR and XOR
- Switch statements
- Discard statement
- Array operations (when arrays are implemented)
