# Replace Math Libcalls with Fixed-Point Implementations

## Overview

Modify the fixed-point transformation pass to detect math library calls (sinf, cosf, tanf, etc.) and replace them with inline fixed-point implementations using pure integer arithmetic. This eliminates the need for floating-point hardware or external libraries on riscv32 targets.

## Architecture

### Current Flow

1. Codegen generates: `call sinf(f32) -> f32`
2. Fixed-point transform converts: fixed-point → float → call sinf → float → fixed-point
3. **Problem**: Requires FPU or libc, doesn't work for binary compilation

### New Flow (Option B)

1. Codegen generates: `call sinf(f32) -> f32` (unchanged)
2. Fixed-point transform detects math libcall and replaces with inline fixed-point implementation
3. **Result**: Pure integer arithmetic, no external dependencies

## Implementation Plan

### 1. Create Fixed-Point Math Implementation Module

**File**: `crates/lp-glsl/src/transform/fixed_point_math.rs` (new)

Create a module that generates fixed-point math function implementations as Cranelift IR sequences. Each function will:

- Take fixed-point input (i32 for Fixed16x16, i64 for Fixed32x32)
- Return fixed-point output
- Use pure integer arithmetic (no floating-point operations)

**Functions to implement**:

- `sin_fixed16x16(i32) -> i32` and `sin_fixed32x32(i64) -> i64`
- `cos_fixed16x16(i32) -> i32` and `cos_fixed32x32(i64) -> i64`
- `tan_fixed16x16(i32) -> i32` and `tan_fixed32x32(i64) -> i64`
- `asin_fixed16x16(i32) -> i32` and `asin_fixed32x32(i64) -> i64`
- `acos_fixed16x16(i32) -> i32` and `acos_fixed32x32(i64) -> i64`
- `atan_fixed16x16(i32) -> i32` and `atan_fixed32x32(i64) -> i64`
- `atan2_fixed16x16(i32, i32) -> i32` and `atan2_fixed32x32(i64, i64) -> i64`
- Hyperbolic functions (sinh, cosh, tanh, asinh, acosh, atanh) - same pattern

**Algorithm choice**: Use CORDIC (COordinate Rotation DIgital Computer) algorithm for production-quality implementation:

- Uniform precision across entire input range (~1 bit per iteration)
- Well-suited for fixed-point arithmetic (only add, subtract, bit-shifts)
- Small memory footprint (precomputed angle table ~32-64 entries)
- Production-quality accuracy suitable for graphics/shaders
- No interpolation needed, no convergence issues

**Key functions**:

```rust
/// Generate inline IR for sin(x) where x is fixed-point
fn generate_sin_fixed(
    cursor: &mut FuncCursor,
    x: Value,
    format: FixedPointFormat,
) -> Value;

/// Generate inline IR for cos(x) where x is fixed-point
fn generate_cos_fixed(
    cursor: &mut FuncCursor,
    x: Value,
    format: FixedPointFormat,
) -> Value;

// ... similar for other functions
```

### 2. Update convert_call to Detect and Replace Math Functions

**File**: `crates/lp-glsl/src/transform/fixed_point.rs`

Modify `convert_call` function (lines 577-686) to:

1. Extract function name from `func.dfg.ext_funcs[func_ref].name`
2. Check if it's a math function we support (sinf, cosf, tanf, etc.)
3. If yes, replace the call with inline fixed-point implementation
4. If no, fall back to current behavior (convert fixed-point → float → call → float → fixed-point)

**Function name mapping**:

- `sinf` → `generate_sin_fixed`
- `cosf` → `generate_cos_fixed`
- `tanf` → `generate_tan_fixed` (or sin/cos division)
- `asinf` → `generate_asin_fixed`
- `acosf` → `generate_acos_fixed`
- `atanf` → `generate_atan_fixed`
- `atan2f` → `generate_atan2_fixed`
- Hyperbolic functions follow same pattern

**Key changes**:

```rust
fn convert_call(...) -> Result<(), GlslError> {
    // ... existing code to extract func_ref, args, etc. ...

    // Check if this is a math function we can replace
    let ext_func = &func.dfg.ext_funcs[func_ref];
    let func_name = match &ext_func.name {
        cranelift_codegen::ir::ExternalName::User(name_ref) => {
            // Extract name from UserExternalName
            // This requires accessing func.params.user_named_funcs()
            // or storing name mapping elsewhere
        }
        _ => return Ok(()), // Not a user function, skip
    };

    // Map libc function name to fixed-point implementation
    if let Some(math_func) = get_fixed_point_math_function(func_name) {
        // Replace call with inline fixed-point implementation
        let fixed_val = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let result = math_func(cursor, fixed_val, format)?;
        value_map.insert(old_results[0], result);
        // Remove old call instruction
        return Ok(());
    }

    // Fall back to existing conversion logic
    // ... existing code ...
}
```

### 3. Implement Fixed-Point Math Functions

**File**: `crates/lp-glsl/src/transform/fixed_point_math.rs`

Implement CORDIC algorithm for trigonometric functions:

**CORDIC Algorithm Overview**:

CORDIC uses iterative rotations to compute trigonometric functions. Each iteration:

1. Determines rotation direction (based on target angle)
2. Rotates current vector by precomputed angle
3. Updates angle accumulator
4. Uses only add, subtract, and bit-shift operations

**For sin(x) and cos(x)** (x in radians, fixed-point):

- **Range reduction**: Reduce x to [0, π/2] using trigonometric identities
- **CORDIC rotation mode**: Start with vector (1, 0) and rotate by angle x
- **Iterations**:
  - Fixed16x16: ~16 iterations for 16-bit precision
  - Fixed32x32: ~32 iterations for 32-bit precision
- **Precomputed angles**: Store atan(2^-i) for i=0..N in fixed-point format
- **Result**: After rotation, sin(x) = y-coordinate, cos(x) = x-coordinate

**For tan(x)**:

- tan(x) = sin(x) / cos(x) using fixed-point division
- Or compute directly using CORDIC vectoring mode

**For atan(x)**:

- **CORDIC vectoring mode**: Rotate vector (x, 1) to x-axis
- Accumulated rotation angle is atan(x)
- Valid for all x values

**For atan2(y, x)**:

- Use CORDIC vectoring mode with vector (x, y)
- Handles all quadrants correctly

**For asin(x) and acos(x)**:

- Use identity: asin(x) = atan(x / sqrt(1 - x²))
- Requires sqrt (can use CORDIC or existing sqrt implementation)
- Clamp input to [-1, 1] range

**Key implementation details**:

- **Precomputed angle table**: Generate at compile time, store in constant pool
  - Fixed16x16: 16 angles (atan(2^-0) through atan(2^-15)) in fixed-point format
  - Fixed32x32: 32 angles (atan(2^-0) through atan(2^-31)) in fixed-point format
  - Store as `ConstantData` in `func.dfg.constants`
- **CORDIC gain factor**:
  - K = ∏(1/√(1 + 2^(-2i))) for i=0..N-1
  - For rotation mode: multiply final result by 1/K (or divide by K)
  - For vectoring mode: multiply final result by K
  - Precompute K in fixed-point format
- **Range reduction for sin/cos**:
  - Reduce input to [0, π/2] using identities:
    - sin(x) = sin(π - x) for x in [π/2, π]
    - sin(x) = -sin(x - π) for x in [π, 2π]
    - cos(x) = -cos(π - x) for x in [π/2, π]
    - cos(x) = cos(2π - x) for x in [π, 2π]
  - Handle sign based on quadrant
- **Fixed-point scaling**:
  - All angles in radians, converted to fixed-point
  - All intermediate values in fixed-point format
  - Use wider intermediate types if needed (i64 for Fixed16x16, i128 for Fixed32x32)
- **Iteration count**:
  - Determines precision (1 bit per iteration)
  - Fixed16x16: 16 iterations for ~16-bit precision
  - Fixed32x32: 32 iterations for ~32-bit precision
- **CORDIC rotation mode (for sin/cos)**:
  ```
  x[0] = K, y[0] = 0, z[0] = angle
  for i = 0 to N-1:
      d = sign(z[i])  // direction: +1 or -1
      x[i+1] = x[i] - d * (y[i] >> i)
      y[i+1] = y[i] + d * (x[i] >> i)
      z[i+1] = z[i] - d * angle_table[i]
  result: sin = y[N] / K, cos = x[N] / K
  ```
- **CORDIC vectoring mode (for atan/atan2)**:
  ```
  x[0] = x_input, y[0] = y_input, z[0] = 0
  for i = 0 to N-1:
      d = sign(y[i])  // rotate towards x-axis
      x[i+1] = x[i] - d * (y[i] >> i)
      y[i+1] = y[i] + d * (x[i] >> i)
      z[i+1] = z[i] - d * angle_table[i]
  result: atan2(y_input, x_input) = z[N]
  ```

### 4. Handle Function Name Extraction

**Challenge**: Cranelift's `ExternalName::User` doesn't directly store the string name.

**Solution options**:

- **Option A**: Store function name mapping during codegen (modify `get_math_libcall` to store name)
- **Option B**: Use function signature to identify (sinf/cosf have unique signatures)
- **Option C**: Use a naming convention in `UserExternalName` that encodes the function name

**Recommended**: Option B - use signature matching:

- sinf/cosf/tanf: `(f32) -> f32`
- atan2f: `(f32, f32) -> f32`
- This is more robust and doesn't require storing names

### 5. Testing Strategy

1. **Unit tests**: Test fixed-point math functions independently

   - Test sin/cos with known values (0, π/2, π, etc.)
   - Verify precision matches expected fixed-point precision

2. **Integration tests**: Update existing GLSL filetests

   - `sin_pi_2.glsl` should work without libc
   - Add tests for various angles and edge cases

3. **Precision validation**: Compare results with float versions

   - Fixed16x16 should match f32 precision (~16 bits)
   - Fixed32x32 should match f64 precision (~32 bits)

## Implementation Order

1. **Phase 1**: Create `fixed_point_math.rs` module with CORDIC infrastructure

   - Generate precomputed angle tables
   - Implement core CORDIC rotation/vectoring logic
   - Implement sin/cos using CORDIC rotation mode

2. **Phase 2**: Update `convert_call` to detect and replace sinf/cosf calls

   - Test with existing GLSL filetests
   - Verify accuracy matches expected fixed-point precision

3. **Phase 3**: Add remaining trigonometric functions

   - tan (using sin/cos division)
   - atan and atan2 (using CORDIC vectoring mode)
   - asin and acos (using atan with sqrt)

4. **Phase 4**: Add hyperbolic functions (sinh, cosh, tanh, asinh, acosh, atanh)
   - Can use CORDIC hyperbolic mode or implement via exponential functions
   - May require additional precomputed tables

## Files to Modify

- `crates/lp-glsl/src/transform/fixed_point.rs` - Update `convert_call` function
- `crates/lp-glsl/src/transform/mod.rs` - Add `fixed_point_math` module
- `crates/lp-glsl/src/transform/fixed_point_math.rs` - New file with math implementations

## Dependencies

- No new external dependencies
- Uses existing Cranelift IR building APIs (`FuncCursor`, `InstBuilder`)
- Uses existing fixed-point conversion utilities

## Success Criteria

### Phase 1 Success (sin/cos implementation)
- ✅ All trigonometric math tests pass for `riscv32.fixed32` and `riscv32.fixed64` targets:
  - `sin_pi_2.glsl` - sin(π/2) ≈ 1.0
  - `sin_scalar.glsl` - basic sin tests
  - `cos_pi.glsl` - cos(π) ≈ -1.0
  - `cos_scalar.glsl` - basic cos tests
  - `sin_vec3.glsl` - vector sin operations
  - `cos_vec2.glsl` - vector cos operations
- ✅ Tests compile without errors (no missing external function errors)
- ✅ Tests run successfully in emulator (no runtime errors)
- ✅ Results match expected values within fixed-point precision tolerances
- ✅ No floating-point operations in generated IR (pure integer arithmetic)

### Phase 2 Success (tan, atan, atan2)
- ✅ All inverse trigonometric tests pass:
  - `tan_scalar.glsl`, `tan_vec3.glsl`
  - `atan_scalar.glsl`
  - `atan2_scalar.glsl`, `atan2_quadrant.glsl`
- ✅ All tests compile and run on riscv32 fixed-point targets

### Phase 3 Success (asin, acos)
- ✅ All remaining trigonometric tests pass:
  - `asin_scalar.glsl`, `asin_boundary.glsl`
  - `acos_scalar.glsl`
- ✅ Edge cases handled correctly (boundary values, domain restrictions)

### Phase 4 Success (hyperbolic functions)
- ✅ All hyperbolic function tests pass:
  - `sinh_scalar.glsl`, `cosh_scalar.glsl`, `tanh_scalar.glsl`
  - `asinh_scalar.glsl`, `acosh_scalar.glsl`, `atanh_scalar.glsl`

### Overall Success Criteria
- ✅ **All math tests pass**: `cargo test --package lp-glsl-filetests` passes for riscv32 fixed-point targets
- ✅ **No external dependencies**: Tests run without libc or floating-point hardware
- ✅ **Production quality**: Accuracy matches or exceeds fixed-point format precision
- ✅ **Performance**: Functions execute in reasonable time (CORDIC iterations are fast)
- ✅ **Code quality**: Clean, maintainable implementation with proper error handling

## Notes

- **Production-quality approach**: CORDIC provides uniform precision across entire input range
- **Fixed-point friendly**: CORDIC uses only integer add/subtract/shift operations, perfect for fixed-point
- **Memory efficient**: Small precomputed angle table (~32-64 entries) vs large lookup tables
- **Precision**: Each iteration adds ~1 bit of precision; 16 iterations for Fixed16x16, 32 for Fixed32x32
- **Range reduction**: Critical for sin/cos to reduce input to [0, π/2] before CORDIC
- **Gain compensation**: CORDIC has constant gain factor that must be compensated in final result
- **Separate implementations**: Fixed16x16 and Fixed32x32 need different iteration counts and angle tables
- **Performance**: Iterative but each iteration is very fast (add/sub/shift only)
