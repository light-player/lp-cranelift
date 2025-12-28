// test run
// target riscv32.fixed32

// ============================================================================
// Phase 6: Testing and Validation
// Acceptance test: Verify all phases work together and dependency pruning works
// ============================================================================

// This test verifies that multiple intrinsic functions from different
// categories can be used together, and that the system correctly handles
// loading multiple intrinsic files.

float test_all_trig_together() {
    // Use multiple trig functions together
    float s = sin(1.5707963267948966); // π/2
    float c = cos(0.0);
    float t = tan(0.7853981633974483); // π/4
    return s + c + t;
}

// run: test_all_trig_together() ~= 3.0

float test_trig_and_exponential_together() {
    // Use trig and exponential functions together (if exponential is implemented)
    float trig = sin(0.0);
    float exp_val = exp(0.0);
    return trig + exp_val;
}

// run: test_trig_and_exponential_together() ~= 1.0

vec3 test_multiple_categories() {
    // Test that functions from multiple categories work together
    vec3 result;
    result.x = sin(0.0);        // trig
    result.y = exp(0.0);        // exponential (if implemented)
    result.z = sin(1.5707963267948966); // trig again
    return result;
}

// run: test_multiple_categories() ~= vec3(0.0, 1.0, 1.0)

// Note: Dependency pruning is verified manually by checking that only
// needed functions are compiled when specific functions are called.
// This test verifies the system works end-to-end.

