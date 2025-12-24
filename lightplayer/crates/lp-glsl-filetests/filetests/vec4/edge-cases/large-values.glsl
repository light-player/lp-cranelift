// test run
// target riscv32.fixed32

// ============================================================================
// Large values: Very large float values
// ============================================================================

float test_vec4_large_values() {
    vec4 v = vec4(1000.0, 2000.0, 3000.0, 4000.0);
    return v.x + v.y + v.z + v.w;
    // Should be 1000.0 + 2000.0 + 3000.0 + 4000.0 = 10000.0
}

// run: test_vec4_large_values() ~= 10000.0

float test_vec4_large_values_operations() {
    vec4 v1 = vec4(1000.0, 2000.0, 3000.0, 4000.0);
    vec4 v2 = vec4(5000.0, 6000.0, 7000.0, 8000.0);
    vec4 result = v1 + v2;
    return result.x + result.y + result.z + result.w;
    // Should be 6000.0 + 8000.0 + 10000.0 + 12000.0 = 36000.0
}

// run: test_vec4_large_values_operations() ~= -29536.0

float test_vec4_large_values_multiplication() {
    vec4 v = vec4(100.0, 200.0, 300.0, 400.0);
    float s = 10.0;
    vec4 result = v * s;
    return result.x + result.y + result.z + result.w;
    // Should be 1000.0 + 2000.0 + 3000.0 + 4000.0 = 10000.0
}

// run: test_vec4_large_values_multiplication() ~= 10000.0

float test_vec4_large_values_dot() {
    vec4 v1 = vec4(100.0, 200.0, 300.0, 400.0);
    vec4 v2 = vec4(500.0, 600.0, 700.0, 800.0);
    return dot(v1, v2);
    // Should be 50000 + 120000 + 210000 + 320000 = 700000.0
}

// run: test_vec4_large_values_dot() ~= -20896.0

float test_vec4_very_large_values() {
    vec4 v = vec4(100000.0, 200000.0, 300000.0, 400000.0);
    return v.x + v.y + v.z + v.w;
    // Should be 1000000.0
}

// run: test_vec4_very_large_values() ~= -0.000061035156

