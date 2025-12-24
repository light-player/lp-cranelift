// test run
// target riscv32.fixed32

// ============================================================================
// Unit vectors: vec4(1.0, 0.0, 0.0, 0.0), etc. - single component = 1.0
// ============================================================================

float test_vec4_unit_x() {
    vec4 v = vec4(1.0, 0.0, 0.0, 0.0);
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 0.0 + 0.0 + 0.0 = 1.0
}

// run: test_vec4_unit_x() ~= 1.0

float test_vec4_unit_y() {
    vec4 v = vec4(0.0, 1.0, 0.0, 0.0);
    return v.x + v.y + v.z + v.w;
    // Should be 0.0 + 1.0 + 0.0 + 0.0 = 1.0
}

// run: test_vec4_unit_y() ~= 1.0

float test_vec4_unit_z() {
    vec4 v = vec4(0.0, 0.0, 1.0, 0.0);
    return v.x + v.y + v.z + v.w;
    // Should be 0.0 + 0.0 + 1.0 + 0.0 = 1.0
}

// run: test_vec4_unit_z() ~= 1.0

float test_vec4_unit_w() {
    vec4 v = vec4(0.0, 0.0, 0.0, 1.0);
    return v.x + v.y + v.z + v.w;
    // Should be 0.0 + 0.0 + 0.0 + 1.0 = 1.0
}

// run: test_vec4_unit_w() ~= 1.0

float test_vec4_unit_length() {
    vec4 v = vec4(1.0, 0.0, 0.0, 0.0);
    return length(v);
    // Length of unit vector should be 1.0
}

// run: test_vec4_unit_length() ~= 1.0

float test_vec4_unit_dot() {
    vec4 v1 = vec4(1.0, 0.0, 0.0, 0.0);
    vec4 v2 = vec4(0.0, 1.0, 0.0, 0.0);
    return dot(v1, v2);
    // Dot product of orthogonal unit vectors should be 0.0
}

// run: test_vec4_unit_dot() ~= 0.0

float test_vec4_unit_self_dot() {
    vec4 v = vec4(0.0, 0.0, 1.0, 0.0);
    return dot(v, v);
    // Dot product of unit vector with itself should be 1.0
}

// run: test_vec4_unit_self_dot() ~= 1.0

