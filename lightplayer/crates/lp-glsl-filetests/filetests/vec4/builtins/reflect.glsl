// test run
// target riscv32.fixed32

// ============================================================================
// Reflect: reflect(vec4, vec4) - reflection vector
// reflect(I, N) = I - 2 * dot(N, I) * N
// ============================================================================

float test_vec4_reflect_simple() {
    vec4 I = vec4(1.0, 0.0, 0.0, 0.0);  // Incident vector
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);  // Normal vector
    vec4 result = reflect(I, N);
    // Reflect off horizontal surface
    // dot(N, I) = 0, so reflect = I - 0 = I = (1, 0, 0, 0)
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 0.0 + 0.0 + 0.0 = 1.0
}

// run: test_vec4_reflect_simple() ~= 1.0

float test_vec4_reflect_45_degree() {
    vec4 I = vec4(1.0, -1.0, 0.0, 0.0);  // Incident at 45 degrees
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);   // Normal pointing up
    vec4 result = reflect(I, N);
    // Reflect off horizontal surface
    // dot(N, I) = -1, so reflect = I - 2*(-1)*N = I + 2*N = (1, -1, 0, 0) + (0, 2, 0, 0) = (1, 1, 0, 0)
    return result.x + result.y + result.z + result.w;
    // Should be approximately 1.0 + 1.0 + 0.0 + 0.0 = 2.0
}

// run: test_vec4_reflect_45_degree() ~= 2.0

float test_vec4_reflect_verify_length() {
    vec4 I = vec4(3.0, 4.0, 0.0, 0.0);
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);
    vec4 result = reflect(I, N);
    // Reflected vector should have same length as incident
    float len_I = length(I);
    float len_R = length(result);
    if (len_I == len_R) {
        return 1.0;
    }
    return 0.0;
    // Should be 1.0 (length preserved)
}

// run: test_vec4_reflect_verify_length() ~= 1.0

