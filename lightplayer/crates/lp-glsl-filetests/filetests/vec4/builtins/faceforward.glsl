// test run
// target riscv32.fixed32

// ============================================================================
// Faceforward: faceforward(vec4, vec4, vec4) - face forward vector
// faceforward(N, I, Nref) = dot(Nref, I) < 0 ? N : -N
// ============================================================================

float test_vec4_faceforward_positive_dot() {
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);      // Normal
    vec4 I = vec4(0.0, 1.0, 0.0, 0.0);      // Incident (same direction as N)
    vec4 Nref = vec4(0.0, 1.0, 0.0, 0.0);   // Reference normal
    vec4 result = faceforward(N, I, Nref);
    // dot(Nref, I) = 1 > 0, so return -N = (0, -1, 0, 0)
    return result.x + result.y + result.z + result.w;
    // Should be 0.0 + -1.0 + 0.0 + 0.0 = -1.0
}

// run: test_vec4_faceforward_positive_dot() ~= -1.0

float test_vec4_faceforward_negative_dot() {
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);      // Normal
    vec4 I = vec4(0.0, -1.0, 0.0, 0.0);     // Incident (opposite to N)
    vec4 Nref = vec4(0.0, 1.0, 0.0, 0.0);   // Reference normal
    vec4 result = faceforward(N, I, Nref);
    // dot(Nref, I) = -1 < 0, so return N = (0, 1, 0, 0)
    return result.x + result.y + result.z + result.w;
    // Should be 0.0 + 1.0 + 0.0 + 0.0 = 1.0
}

// run: test_vec4_faceforward_negative_dot() ~= 1.0

float test_vec4_faceforward_zero_dot() {
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);      // Normal
    vec4 I = vec4(1.0, 0.0, 0.0, 0.0);      // Incident (perpendicular)
    vec4 Nref = vec4(0.0, 1.0, 0.0, 0.0);   // Reference normal
    vec4 result = faceforward(N, I, Nref);
    // dot(Nref, I) = 0, so return -N = (0, -1, 0, 0)
    return result.x + result.y + result.z + result.w;
    // Should be 0.0 + -1.0 + 0.0 + 0.0 = -1.0
}

// run: test_vec4_faceforward_zero_dot() ~= -1.0

float test_vec4_faceforward_verify_logic() {
    vec4 N = vec4(1.0, 0.0, 0.0, 0.0);
    vec4 I1 = vec4(1.0, 0.0, 0.0, 0.0);   // Same direction
    vec4 I2 = vec4(-1.0, 0.0, 0.0, 0.0);  // Opposite direction
    vec4 Nref = vec4(1.0, 0.0, 0.0, 0.0);
    vec4 r1 = faceforward(N, I1, Nref);
    vec4 r2 = faceforward(N, I2, Nref);
    // r1 should be -N, r2 should be N
    if (r1.x == -1.0 && r2.x == 1.0) {
        return 1.0;
    }
    return 0.0;
    // Should be 1.0 (logic correct)
}

// run: test_vec4_faceforward_verify_logic() ~= 1.0

