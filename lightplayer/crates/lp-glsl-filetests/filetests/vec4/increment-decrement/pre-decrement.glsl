// test run
// target riscv32.fixed32

// ============================================================================
// Pre-decrement: --v (decrements then returns)
// ============================================================================

float test_vec4_pre_decrement() {
    vec4 v = vec4(3.0, 4.0, 5.0, 6.0);
    vec4 result = --v;
    // Should decrement v to (2.0, 3.0, 4.0, 5.0), then return (2.0, 3.0, 4.0, 5.0)
    return result.x + result.y + result.z + result.w;
    // Should be 2.0 + 3.0 + 4.0 + 5.0 = 14.0
}

// run: test_vec4_pre_decrement() ~= 14.0

float test_vec4_pre_decrement_verify_value() {
    vec4 v = vec4(3.0, 4.0, 5.0, 6.0);
    --v;
    // Verify v was decremented
    return v.x + v.y + v.z + v.w;
    // Should be 2.0 + 3.0 + 4.0 + 5.0 = 14.0
}

// run: test_vec4_pre_decrement_verify_value() ~= 14.0

float test_vec4_pre_decrement_verify_components() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = --v;
    // Verify each component decremented
    float sum = 0.0;
    if (result.x == 9.0) sum = sum + 1.0;
    if (result.y == 19.0) sum = sum + 1.0;
    if (result.z == 29.0) sum = sum + 1.0;
    if (result.w == 39.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components decremented correctly)
}

// run: test_vec4_pre_decrement_verify_components() ~= 4.0

float test_vec4_pre_decrement_negative() {
    vec4 v = vec4(-1.0, -2.0, -3.0, -4.0);
    vec4 result = --v;
    // Decrement negative values
    return result.x + result.y + result.z + result.w;
    // Should be -2.0 + -3.0 + -4.0 + -5.0 = -14.0
}

// run: test_vec4_pre_decrement_negative() ~= -14.0

float test_vec4_pre_decrement_decimal() {
    vec4 v = vec4(1.5, 2.5, 3.5, 4.5);
    vec4 result = --v;
    // Decrement decimal values
    return result.x + result.y + result.z + result.w;
    // Should be 0.5 + 1.5 + 2.5 + 3.5 = 8.0
}

// run: test_vec4_pre_decrement_decimal() ~= 8.0

