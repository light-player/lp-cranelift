// test run
// target riscv32.fixed32

// ============================================================================
// Dot product: dot(vec4, vec4) - component-wise multiplication and sum
// ============================================================================

float test_vec4_dot() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(5.0, 6.0, 7.0, 8.0);
    return dot(a, b);
    // Should be 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70.0
}

// run: test_vec4_dot() ~= 70.0

float test_vec4_dot_zero() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(0.0, 0.0, 0.0, 0.0);
    return dot(a, b);
    // Should be 0.0 (dot with zero vector)
}

// run: test_vec4_dot_zero() ~= 0.0

float test_vec4_dot_orthogonal() {
    vec4 a = vec4(1.0, 0.0, 0.0, 0.0);
    vec4 b = vec4(0.0, 1.0, 0.0, 0.0);
    return dot(a, b);
    // Should be 0.0 (orthogonal vectors)
}

// run: test_vec4_dot_orthogonal() ~= 0.0

float test_vec4_dot_self() {
    vec4 v = vec4(3.0, 4.0, 0.0, 0.0);
    return dot(v, v);
    // Should be 3*3 + 4*4 + 0*0 + 0*0 = 9 + 16 = 25.0
}

// run: test_vec4_dot_self() ~= 25.0

float test_vec4_dot_negative() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(-1.0, -2.0, -3.0, -4.0);
    return dot(a, b);
    // Should be -1 + -4 + -9 + -16 = -30.0
}

// run: test_vec4_dot_negative() ~= -30.0

float test_vec4_dot_mixed() {
    vec4 a = vec4(1.0, -2.0, 3.0, -4.0);
    vec4 b = vec4(-5.0, 6.0, -7.0, 8.0);
    return dot(a, b);
    // Should be -5 + -12 + -21 + -32 = -70.0
}

// run: test_vec4_dot_mixed() ~= -70.0

