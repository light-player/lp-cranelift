---
name: math-tests-emulator-validation
overview: Ensure all math function tests in filetests use riscv32 emulator execution with fixed-point formats to validate correctness
todos:
  - id: audit_math_tests
    content: Audit all math-related test files and identify which need emulator targets
    status: pending
  - id: add_target_directives
    content: Add target directives to math tests (riscv32.fixed32 and riscv32.fixed64)
    status: pending
  - id: verify_binary_compilation
    content: Verify binary compilation works for all math test cases
    status: pending
  - id: test_trigonometric_functions
    content: Ensure trigonometric functions (sin, cos, tan, etc.) work in emulator
    status: pending
  - id: test_common_functions
    content: Ensure common functions (sqrt, pow, abs, etc.) work in emulator
    status: pending
  - id: test_geometric_functions
    content: Ensure geometric functions (dot, cross, length, etc.) work in emulator
    status: pending
  - id: validate_fixed_point_precision
    content: Validate that fixed-point conversion maintains acceptable precision for math functions
    status: pending
---

# Math Tests Emulator Validation Plan

## Overview

Ensure all math function tests in `crates/lp-glsl-filetests/filetests` use the new binary compilation and riscv32 emulator execution to validate correctness, especially with fixed-point formats (16.16 and 32.32).

## Current State

### Known Issues

1. **JIT Initialization**: Tests that use `test compile` directive will fail for riscv32 targets because `test_compile` uses `JIT::new()` which requires native architecture support. For now, riscv32 tests should only use `test run`.

2. **Vector/Matrix Return Types**: Currently only `FloatApprox` return type supports riscv32 emulator execution. Other return types (Vec2, Vec3, Vec4, Mat2, Mat3, Mat4, Int, Bool) still use JIT even for riscv32 targets. These need to be updated to use the execution backend abstraction.

3. **Test Infrastructure**: The test infrastructure needs to be updated to handle riscv32 targets properly for all return types.

### Math Test Categories

1. **Trigonometric Functions** (`builtins/trigonometric/`, `builtins/sin_*.glsl`, `builtins/cos_*.glsl`, `builtins/tan_*.glsl`)
   - sin, cos, tan
   - asin, acos, atan, atan2
   - sinh, cosh, tanh
   - asinh, acosh, atanh
   - radians, degrees

2. **Common Functions** (`builtins/common/`)
   - abs, sign
   - floor, ceil, fract, mod
   - min, max, clamp
   - sqrt, pow

3. **Geometric Functions** (`builtins/geometric/`)
   - length, distance
   - dot, cross
   - normalize

4. **Interpolation Functions** (`builtins/interpolation/`)
   - mix, smoothstep, step

5. **Matrix Functions** (`builtins/matrix/`)
   - transpose, inverse, determinant

### Current Test Structure

Most math tests currently:
- Use `// test compile` and `// test run` directives
- Run only on host (native JIT)
- Don't specify targets or fixed-point formats
- Use explicit tolerance values (e.g., `tolerance: 0.001`)

Example:
```glsl
// test compile
// test run

float main() {
    return sin(0.0);
}

// run: ~= 0.0 (tolerance: 0.001)
```

## Goals

1. **Add Target Directives**: All math tests should specify `target riscv32.fixed32` and/or `target riscv32.fixed64`
2. **Remove Explicit Tolerances**: Use default tolerances based on target (already implemented)
3. **Validate Fixed-Point Precision**: Ensure math functions maintain acceptable precision in fixed-point formats
4. **Comprehensive Coverage**: Test all math functions in emulator with both fixed-point formats

## Implementation Plan

### Phase 1: Verify Binary Compilation Works

**Tasks:**
1. Create a simple test case to verify binary compilation works
2. Test with a basic math function (e.g., `sqrt`)
3. Verify emulator execution produces correct results
4. Test with both fixed32 and fixed64 formats

**Test Case:**
```glsl
// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return sqrt(4.0);
}

// run: ~= 2.0
```

**Verification Steps:**
- Compile GLSL to binary
- Execute in riscv32 emulator
- Verify result matches expected value
- Check that tolerance defaults work correctly

### Phase 2: Audit Math Tests

**Tasks:**
1. List all math-related test files
2. Categorize by function type
3. Identify which tests need emulator targets
4. Document current test coverage

**Math Test Files to Update:**

**Trigonometric:**
- `builtins/sin_scalar.glsl`
- `builtins/sin_pi_2.glsl`
- `builtins/sin_vec3.glsl`
- `builtins/cos_scalar.glsl`
- `builtins/cos_pi.glsl`
- `builtins/cos_vec2.glsl`
- `builtins/tan_scalar.glsl`
- `builtins/tan_vec3.glsl`
- `builtins/sinh_scalar.glsl`
- `builtins/cosh_scalar.glsl`
- `builtins/tanh_scalar.glsl`
- `builtins/asin_scalar.glsl`
- `builtins/asin_boundary.glsl`
- `builtins/acos_scalar.glsl`
- `builtins/atan_scalar.glsl`
- `builtins/atan2_scalar.glsl`
- `builtins/atan2_quadrant.glsl`
- `builtins/asinh_scalar.glsl`
- `builtins/acosh_scalar.glsl`
- `builtins/atanh_scalar.glsl`
- `builtins/radians_scalar.glsl`
- `builtins/radians_vec2.glsl`
- `builtins/degrees_scalar.glsl`
- `builtins/radians_degrees_roundtrip.glsl`
- All files in `builtins/trigonometric/` (24 files)

**Common Functions:**
- `builtins/common/abs_float.glsl`
- `builtins/common/abs_vec2.glsl`
- `builtins/common/sign_float.glsl`
- `builtins/common/sign_int.glsl`
- `builtins/common/floor_scalar.glsl`
- `builtins/common/floor_vec2.glsl`
- `builtins/common/ceil_scalar.glsl`
- `builtins/common/ceil_vec3.glsl`
- `builtins/common/fract_scalar.glsl`
- `builtins/common/fract_vec2.glsl`
- `builtins/common/mod_scalar.glsl`
- `builtins/common/mod_vec3.glsl`
- `builtins/common/mod_vec3_scalar.glsl`
- `builtins/common/min_scalars.glsl`
- `builtins/common/min_vec3_scalar.glsl`
- `builtins/common/max_vec2.glsl`
- `builtins/common/clamp_scalar.glsl`
- `builtins/common/clamp_vec3.glsl`
- `builtins/common/sqrt_scalar.glsl`
- `builtins/common/sqrt_vec3.glsl`
- `builtins/common/pow_error.glsl` (error test, skip)

**Geometric Functions:**
- `builtins/geometric/length_vec2.glsl`
- `builtins/geometric/distance_vec3.glsl`
- `builtins/geometric/dot_vec3.glsl`
- `builtins/geometric/cross_vec3.glsl`
- `builtins/geometric/normalize_vec3.glsl`

**Interpolation:**
- All files in `builtins/interpolation/` (9 files)

**Matrix Functions:**
- All files in `builtins/matrix/` (6 files)

### Phase 3: Add Target Directives

**Strategy:**
1. For each math test file:
   - Add `// target riscv32.fixed32` directive
   - Add `// target riscv32.fixed64` directive (for functions that need higher precision)
   - Remove explicit `tolerance:` specifiers (use defaults)
   - Keep `// test compile` and `// test run` directives

2. **Precision Guidelines:**
   - **fixed32 (16.16)**: Use for functions that can tolerate ~0.001 precision
     - Basic arithmetic
     - Simple functions (abs, sign, floor, ceil)
     - Some trigonometric functions (if precision acceptable)
   - **fixed64 (32.32)**: Use for functions requiring higher precision
     - Trigonometric functions (sin, cos, tan)
     - Inverse trigonometric functions (asin, acos, atan)
     - Hyperbolic functions
     - sqrt, pow
     - Geometric functions (length, distance, normalize)

3. **Example Transformation:**

**Before:**
```glsl
// test compile
// test run

float main() {
    return sin(0.0);
}

// run: ~= 0.0 (tolerance: 0.001)
```

**After:**
```glsl
// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return sin(0.0);
}

// run: ~= 0.0
```

### Phase 4: Test Execution and Validation

**Tasks:**
1. Run all updated math tests
2. Verify they pass in emulator
3. Check that results match expected values within tolerance
4. Document any precision issues

**Test Execution:**
```bash
cargo test --package lp-glsl-filetests --lib filetest
```

**Validation Checklist:**
- [ ] All trigonometric tests pass in riscv32.fixed32
- [ ] All trigonometric tests pass in riscv32.fixed64
- [ ] All common function tests pass in riscv32.fixed32
- [ ] All common function tests pass in riscv32.fixed64
- [ ] All geometric function tests pass in riscv32.fixed32
- [ ] All geometric function tests pass in riscv32.fixed64
- [ ] Precision is acceptable for all functions
- [ ] No regressions in host execution

### Phase 5: Handle Edge Cases

**Tasks:**
1. Test boundary conditions (e.g., `asin_boundary.glsl`)
2. Test special values (0, pi, pi/2, etc.)
3. Test vector and matrix variants
4. Verify error cases still work (e.g., `pow_error.glsl`)

**Special Considerations:**
- Some tests may need different tolerances for fixed32 vs fixed64
- Vector/matrix tests may need special handling
- Error tests should not have emulator targets (they test compilation errors)

## Implementation Steps

### Step 1: Create Test Script
Create a script to verify binary compilation works:
```bash
#!/bin/bash
# test-binary-compilation.sh
cargo test --package lp-glsl-filetests --lib test_run::run_test -- --nocapture
```

### Step 2: Update Test Files (Automated)
Create a script to add target directives to all math tests:
```bash
#!/bin/bash
# add-targets.sh
for file in crates/lp-glsl-filetests/filetests/builtins/**/*.glsl; do
    # Skip error tests
    if grep -q "test error" "$file"; then
        continue
    fi
    
    # Add targets if not present
    if ! grep -q "target riscv32" "$file"; then
        # Add after "test run" line
        sed -i '' '/test run/a\
// target riscv32.fixed32\
// target riscv32.fixed64
' "$file"
    fi
done
```

### Step 3: Manual Review
Review each test file to:
- Ensure appropriate targets (fixed32 vs fixed64)
- Remove explicit tolerances
- Verify expected values are correct
- Check that test cases are appropriate for emulator execution

### Step 4: Run Tests
Execute all tests and fix any failures:
```bash
cargo test --package lp-glsl-filetests
```

### Step 5: Document Results
Document:
- Which tests pass/fail
- Precision achieved for each function
- Any limitations or known issues
- Recommendations for tolerance adjustments

## Expected Outcomes

1. **All math tests run in riscv32 emulator** with appropriate fixed-point formats
2. **Default tolerances work correctly** for all test cases
3. **Precision is acceptable** for all math functions in fixed-point formats
4. **No regressions** in host execution
5. **Comprehensive coverage** of all math functions

## Success Criteria

- [ ] At least 90% of math tests pass in riscv32.fixed32
- [ ] At least 90% of math tests pass in riscv32.fixed64
- [ ] All critical math functions (sin, cos, sqrt, etc.) work correctly
- [ ] Precision is within acceptable bounds for fixed-point formats
- [ ] Test execution time is reasonable (< 5 minutes for all tests)

## Future Enhancements

1. **Performance Testing**: Measure execution time in emulator vs host
2. **Precision Analysis**: Detailed analysis of precision loss in fixed-point formats
3. **Optimization**: Optimize fixed-point math functions for better precision
4. **Extended Coverage**: Add more test cases for edge conditions
5. **Automated Validation**: Create CI checks to ensure new math tests include emulator targets

## Related Plans

- `.plans/lp-glsl-binary-compilation.md` - Binary compilation implementation (completed)
- `.plans/lp-filetests-emulator-execution.md` - Emulator execution infrastructure (completed)

