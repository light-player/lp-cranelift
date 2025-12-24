// test run
// target riscv32.fixed32

// ============================================================================
// Zero vector: vec4(0.0) - all components zero
// ============================================================================

float test_vec4_zero_vector() {
    vec4 v = vec4(0.0);
    return v.x + v.y + v.z + v.w;
    // Should be 0.0 + 0.0 + 0.0 + 0.0 = 0.0
}

// run: test_vec4_zero_vector() ~= 0.0

float test_vec4_zero_vector_verify_components() {
    vec4 v = vec4(0.0);
    float sum = 0.0;
    if (v.x == 0.0) sum = sum + 1.0;
    if (v.y == 0.0) sum = sum + 1.0;
    if (v.z == 0.0) sum = sum + 1.0;
    if (v.w == 0.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components zero)
}

// run: test_vec4_zero_vector_verify_components() ~= 4.0

float test_vec4_zero_vector_operations() {
    vec4 v = vec4(0.0);
    vec4 result = v + vec4(1.0, 2.0, 3.0, 4.0);
    // Zero vector + other vector
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_zero_vector_operations() ~= 10.0

float test_vec4_zero_vector_dot() {
    vec4 v1 = vec4(0.0);
    vec4 v2 = vec4(1.0, 2.0, 3.0, 4.0);
    return dot(v1, v2);
    // Dot product with zero vector should be 0.0
}

// run: test_vec4_zero_vector_dot() ~= 0.0

float test_vec4_zero_vector_length() {
    vec4 v = vec4(0.0);
    return length(v);
    // Length of zero vector should be 0.0
}

// run: test_vec4_zero_vector_length() ~= 0.0

