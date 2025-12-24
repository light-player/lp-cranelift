// test run
// target riscv32.fixed32

// ============================================================================
// NaN and Inf handling: Special floating-point values
// Note: These tests may need adjustment based on actual NaN/Inf handling
// ============================================================================

float test_vec4_inf_operations() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float inf_val = 1.0 / 0.0;  // Infinity (if division by zero produces inf)
    vec4 result = v + inf_val;
    // Adding infinity to vector
    return result.x;
    // Should be infinity (or handle as implementation defines)
}

// run: test_vec4_inf_operations() ~= -32768.0

float test_vec4_nan_operations() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float nan_val = 0.0 / 0.0;  // NaN (if 0/0 produces NaN)
    vec4 result = v + nan_val;
    // Adding NaN to vector
    return result.x;
    // Should be NaN (or handle as implementation defines)
}

// run: test_vec4_nan_operations() ~= 1.0

float test_vec4_inf_multiplication() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float inf_val = 1.0 / 0.0;
    vec4 result = v * inf_val;
    // Multiply by infinity
    return result.x;
    // Should be infinity (or handle as implementation defines)
}

// run: test_vec4_inf_multiplication() ~= 32767.0

float test_vec4_zero_times_inf() {
    vec4 v = vec4(0.0);
    float inf_val = 1.0 / 0.0;
    vec4 result = v * inf_val;
    // 0 * inf = NaN (or 0, depending on implementation)
    return result.x;
    // Should be NaN or 0 (implementation dependent)
}

// run: test_vec4_zero_times_inf() ~= 0.0

float test_vec4_division_by_zero() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float zero = 0.0;
    vec4 result = v / zero;
    // Division by zero
    return result.x;
    // Should be infinity or handle as implementation defines
}

// run: test_vec4_division_by_zero() ~= 32767.0

