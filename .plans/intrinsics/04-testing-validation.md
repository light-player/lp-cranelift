# Stage 04: Testing and Validation

## Overview

Comprehensive testing and validation of the intrinsic function system, ensuring all components work correctly together in both float and fixed-point modes.

## Goals

- Verify intrinsics compile lazily and are added to module correctly
- Verify intrinsics are converted to fixed-point correctly
- Verify function calls work correctly in both modes
- Add comprehensive test coverage
- Validate performance characteristics

## Testing Strategy

### 1. Unit Tests for Individual Components

#### Test: Intrinsic Compilation

**File**: `crates/lp-glsl/tests/intrinsics_compilation.rs`

```rust
#[test]
fn test_intrinsic_compiles_lazily() {
    // Test that intrinsics are only compiled when needed
}

#[test]
fn test_intrinsic_added_to_module() {
    // Test that compiled intrinsics are added to module
}

#[test]
fn test_intrinsic_cached() {
    // Test that intrinsics are cached and not recompiled
}
```

#### Test: Fixed-Point Conversion

**File**: `crates/lp-glsl/tests/intrinsics_fixed_point.rs`

```rust
#[test]
fn test_intrinsic_converted_to_fixed() {
    // Test that intrinsics are converted when fixed-point enabled
}

#[test]
fn test_intrinsic_no_f32_after_conversion() {
    // Test that no F32 types remain after conversion
}
```

### 2. Integration Tests via Filetests

#### Test: Basic Math Functions

**Files**: `lightplayer/crates/lp-glsl-filetests/filetests/math/`

- `sine.glsl` - Test sin() function
- `cosine.glsl` - Test cos() function
- `tangent.glsl` - Test tan() function
- `arcsine.glsl` - Test asin() function
- `arccosine.glsl` - Test acos() function
- `arctangent.glsl` - Test atan() function

Each test should:

- Test compile (verify intrinsic is used, not external call)
- Test fixed32 (verify fixed-point conversion)
- Test run (verify correctness at key mathematical points)

#### Test: Fixed-Point Math Functions

**Files**: `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/`

- `sin.glsl` - sin() in fixed-point
- `cos.glsl` - cos() in fixed-point
- `tan.glsl` - tan() in fixed-point

### 3. Edge Case Tests

#### Test: Multiple Intrinsics

**File**: `lightplayer/crates/lp-glsl-filetests/filetests/math/multiple.glsl`

```glsl
// test compile
// test run

float main() {
    float x = 1.0;
    float result = sin(x) + cos(x) + tan(x);
    return result;
}

// Verify all three intrinsics are compiled and work together
```

#### Test: Intrinsic Dependencies

**File**: `lightplayer/crates/lp-glsl-filetests/filetests/math/dependencies.glsl`

```glsl
// test compile
// test fixed32

float main() {
    // cos() calls sin() internally, test that works
    return cos(0.0);
}

// Verify dependency handling works correctly
```

#### Test: Precision

**File**: `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/precision.glsl`

```glsl
// test compile
// test fixed32
// test run

float main() {
    // Test precision at various points
    float pi = 3.141592654;
    float pi_2 = 1.570796327;

    float s1 = sin(0.0);        // Should be 0.0
    float s2 = sin(pi_2);       // Should be ~1.0
    float s3 = sin(pi);         // Should be ~0.0
    float s4 = sin(pi * 1.5);   // Should be ~-1.0

    return s1 + s2 + s3 + s4;   // Should be ~0.0
}

// run: ~= 0.0  (tolerance: 0.05 for fixed-point precision)
```

### 4. Performance Tests

#### Test: Lazy Compilation

Verify that only needed intrinsics are compiled:

```rust
#[test]
fn test_only_called_intrinsics_compiled() {
    // Compile shader that calls sin() but not cos()
    // Verify cos() is not compiled
}
```

#### Test: Caching

Verify that intrinsics are cached:

```rust
#[test]
fn test_intrinsic_cached_across_calls() {
    // Call sin() multiple times
    // Verify it's only compiled once
}
```

## Validation Checklist

### Compilation Validation

- [ ] Intrinsics compile lazily (only when needed)
- [ ] Intrinsics are added to module correctly
- [ ] Intrinsics are cached properly
- [ ] Only called intrinsics are compiled
- [ ] Intrinsic dependencies are handled

### Fixed-Point Validation

- [ ] Intrinsics are converted to fixed-point when enabled
- [ ] Intrinsics are converted before main function
- [ ] No F32 types remain after conversion
- [ ] Function signatures are converted correctly
- [ ] Function calls work correctly after conversion

### Correctness Validation

- [ ] sin(0) = 0
- [ ] sin(π/2) ≈ 1
- [ ] sin(π) ≈ 0
- [ ] sin(3π/2) ≈ -1
- [ ] cos(0) = 1
- [ ] cos(π/2) ≈ 0
- [ ] cos(π) ≈ -1
- [ ] Similar checks for other functions

### Integration Validation

- [ ] User functions + intrinsics work together
- [ ] Multiple intrinsics in one shader work
- [ ] Intrinsic dependencies work (cos calling sin)
- [ ] Fixed-point precision is acceptable
- [ ] No regressions in existing functionality

## Test Files to Create

### Math Function Tests

- `lightplayer/crates/lp-glsl-filetests/filetests/math/sine.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/math/cosine.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/math/tangent.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/math/multiple.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/math/dependencies.glsl`

### Fixed-Point Math Tests

- `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/sin.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/cos.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/tan.glsl`
- `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/precision.glsl`

### Unit Tests

- `crates/lp-glsl/tests/intrinsics_compilation.rs`
- `crates/lp-glsl/tests/intrinsics_fixed_point.rs`

## Success Criteria

- [ ] All unit tests pass
- [ ] All filetests pass (compile, fixed32, run)
- [ ] Math functions produce correct results
- [ ] Fixed-point precision is acceptable
- [ ] No performance regressions
- [ ] Code coverage meets targets
- [ ] Documentation is complete

## Files to Create

1. `lightplayer/crates/lp-glsl-filetests/filetests/math/sine.glsl`
2. `lightplayer/crates/lp-glsl-filetests/filetests/math/cosine.glsl`
3. `lightplayer/crates/lp-glsl-filetests/filetests/math/tangent.glsl`
4. `lightplayer/crates/lp-glsl-filetests/filetests/math/multiple.glsl`
5. `lightplayer/crates/lp-glsl-filetests/filetests/math/dependencies.glsl`
6. `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/sin.glsl`
7. `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/cos.glsl`
8. `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/tan.glsl`
9. `lightplayer/crates/lp-glsl-filetests/filetests/fixed32/math/precision.glsl`
10. `crates/lp-glsl/tests/intrinsics_compilation.rs`
11. `crates/lp-glsl/tests/intrinsics_fixed_point.rs`

## Dependencies

- **Depends on**: All previous stages (01, 02, 03)
- **Final stage**: This completes the intrinsic function implementation

## Notes

- Tests should be comprehensive but focused
- Edge cases are important (zero, infinity, NaN handling)
- Performance tests ensure lazy compilation works
- Documentation should explain how to add new intrinsics
