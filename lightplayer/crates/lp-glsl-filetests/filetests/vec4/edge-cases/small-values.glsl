// test run
// target riscv32.fixed32

// ============================================================================
// Small values: Very small float values
// ============================================================================

float test_vec4_small_values() {
    vec4 v = vec4(0.001, 0.002, 0.003, 0.004);
    return v.x + v.y + v.z + v.w;
    // Should be 0.001 + 0.002 + 0.003 + 0.004 = 0.01
}

// run: test_vec4_small_values() ~= 0.01

float test_vec4_small_values_operations() {
    vec4 v1 = vec4(0.001, 0.002, 0.003, 0.004);
    vec4 v2 = vec4(0.005, 0.006, 0.007, 0.008);
    vec4 result = v1 + v2;
    return result.x + result.y + result.z + result.w;
    // Should be 0.006 + 0.008 + 0.01 + 0.012 = 0.036
}

// run: test_vec4_small_values_operations() ~= 0.036

float test_vec4_small_values_multiplication() {
    vec4 v = vec4(0.1, 0.2, 0.3, 0.4);
    float s = 0.5;
    vec4 result = v * s;
    return result.x + result.y + result.z + result.w;
    // Should be 0.05 + 0.1 + 0.15 + 0.2 = 0.5
}

// run: test_vec4_small_values_multiplication() ~= 0.5

float test_vec4_small_values_dot() {
    vec4 v1 = vec4(0.1, 0.2, 0.3, 0.4);
    vec4 v2 = vec4(0.5, 0.6, 0.7, 0.8);
    return dot(v1, v2);
    // Should be 0.05 + 0.12 + 0.21 + 0.32 = 0.7
}

// run: test_vec4_small_values_dot() ~= 0.7

float test_vec4_very_small_values() {
    vec4 v = vec4(0.0001, 0.0002, 0.0003, 0.0004);
    return v.x + v.y + v.z + v.w;
    // Should be 0.001
}

// run: test_vec4_very_small_values() ~= 0.001

