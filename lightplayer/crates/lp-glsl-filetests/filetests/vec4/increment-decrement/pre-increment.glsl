// test run
// target riscv32.fixed32

// ============================================================================
// Pre-increment: ++v (increments then returns)
// ============================================================================

float test_vec4_pre_increment() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = ++v;
    // Should increment v to (2.0, 3.0, 4.0, 5.0), then return (2.0, 3.0, 4.0, 5.0)
    return result.x + result.y + result.z + result.w;
    // Should be 2.0 + 3.0 + 4.0 + 5.0 = 14.0
}

// run: test_vec4_pre_increment() ~= 14.0

float test_vec4_pre_increment_verify_value() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    ++v;
    // Verify v was incremented
    return v.x + v.y + v.z + v.w;
    // Should be 2.0 + 3.0 + 4.0 + 5.0 = 14.0
}

// run: test_vec4_pre_increment_verify_value() ~= 14.0

float test_vec4_pre_increment_verify_components() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = ++v;
    // Verify each component incremented
    float sum = 0.0;
    if (result.x == 11.0) sum = sum + 1.0;
    if (result.y == 21.0) sum = sum + 1.0;
    if (result.z == 31.0) sum = sum + 1.0;
    if (result.w == 41.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components incremented correctly)
}

// run: test_vec4_pre_increment_verify_components() ~= 4.0

float test_vec4_pre_increment_negative() {
    vec4 v = vec4(-1.0, -2.0, -3.0, -4.0);
    vec4 result = ++v;
    // Increment negative values
    return result.x + result.y + result.z + result.w;
    // Should be 0.0 + -1.0 + -2.0 + -3.0 = -6.0
}

// run: test_vec4_pre_increment_negative() ~= -6.0

float test_vec4_pre_increment_decimal() {
    vec4 v = vec4(0.5, 1.5, 2.5, 3.5);
    vec4 result = ++v;
    // Increment decimal values
    return result.x + result.y + result.z + result.w;
    // Should be 1.5 + 2.5 + 3.5 + 4.5 = 12.0
}

// run: test_vec4_pre_increment_decimal() ~= 12.0

