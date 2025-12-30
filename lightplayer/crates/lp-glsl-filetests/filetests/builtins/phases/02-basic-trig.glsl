// test run
// target riscv32.fixed32

// ============================================================================
// Phase 2: Basic Trig Functions (sin, cos, tan)
// Acceptance test: Verify sin, cos, tan work with dependency tracking
// ============================================================================

float test_sin_basic() {
    // Test that sin() works
    return sin(1.5707963267948966); // π/2
}

// run: test_sin_basic() ~= 1.0

float test_cos_basic() {
    // Test that cos() works (should call sin internally)
    return cos(0.0);
}

// run: test_cos_basic() ~= 1.0

float test_tan_basic() {
    // Test that tan() works
    return tan(0.7853981633974483); // π/4
}

// run: test_tan_basic() ~= 1.0

vec2 test_trig_vec2() {
    // Test component-wise operation
    return sin(vec2(0.0, 1.5707963267948966));
}

// run: test_trig_vec2() ~= vec2(0.0, 1.0)

// Note: Dependency tracking is verified by checking that only needed
// functions are compiled. This is verified manually during development
// by inspecting the generated code.



