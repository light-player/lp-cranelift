# GLSL Feature Implementation Plans

## Overview

This directory contains detailed implementation plans for remaining GLSL language features. Each plan focuses on a specific language feature and emphasizes:

- **Comprehensive testing** (functionality and error handling)
- **Good code structure** (modular, maintainable)
- **Spec compliance** (reference to GLSL spec sections)

## Current State

**Completed Features:**
- ✅ Basic types (int, bool, float)
- ✅ Vectors (vec2/3/4, ivec2/3/4, bvec2/3/4)
- ✅ Matrices (mat2/3/4) with operations
- ✅ Control flow (if/else, for, while, break, continue)
- ✅ User-defined functions
- ✅ Basic built-ins (dot, cross, length, normalize, distance, min, max, clamp, abs, sqrt, floor, ceil, mix, step, smoothstep, fract, mod, sign)
- ✅ Matrix built-ins (matrixCompMult, outerProduct, transpose, determinant, inverse)
- ✅ Fixed-point transformation (16.16 and 32.32 formats)

**Remaining Features:**

1. **01-trigonometric-functions.md** - Angle and trigonometry functions (sin, cos, tan, etc.) - **HIGH PRIORITY**
2. **02-exponential-logarithmic.md** - Exponential and logarithmic functions (exp, log, pow, etc.) - **HIGH PRIORITY**
3. **03-struct-types.md** - User-defined struct types with member access - **MEDIUM PRIORITY**
4. **04-array-types.md** - Fixed-size arrays with indexing - **MEDIUM PRIORITY**
5. **05-additional-builtins.md** - Additional common and geometric functions (trunc, round, reflect, etc.) - **LOW PRIORITY**
6. **06-texture-sampling.md** - Texture access functions (requires runtime integration) - **LOW PRIORITY**
7. **07-uniforms-io.md** - Storage qualifiers (uniform, in, out) and shader I/O - **MEDIUM PRIORITY**

## Implementation Priority

1. **High Priority** (needed for common shader effects):
   - Trigonometric functions
   - Exponential/logarithmic functions
   - Struct types

2. **Medium Priority** (useful but can work around):
   - Array types
   - Additional built-ins
   - Uniforms/I/O

3. **Low Priority** (requires runtime):
   - Texture sampling

## Common Patterns

All plans follow these patterns:

### Code Structure
- Separate semantic analysis from code generation
- Use existing pass-based architecture
- Follow existing module organization
- Reuse existing patterns (e.g., builtin registration)

### Testing Strategy
- **Functionality tests**: Verify correct behavior
- **Error handling tests**: Invalid inputs, type mismatches
- **Edge cases**: Boundary conditions, special values
- **Vector component-wise**: Test scalar and vector variants

### Error Messages
- Clear, actionable error messages
- Reference to GLSL spec when appropriate
- Source location information
- Helpful suggestions when possible

## Spec Reference

All plans reference the GLSL specification at:
`/Users/yona/dev/photomancer/glsl-spec/chapters/`

Key chapters:
- `variables.adoc` - Type definitions and declarations
- `builtinfunctions.adoc` - Built-in function specifications
- `operators.adoc` - Operator semantics

