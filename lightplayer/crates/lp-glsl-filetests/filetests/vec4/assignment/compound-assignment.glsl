// test run
// target riscv32.fixed32

// ============================================================================
// Compound assignment: +=, -=, *=, /=
// ============================================================================

float test_vec4_add_assign() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(10.0, 20.0, 30.0, 40.0);
    v += v2;
    // Component-wise addition in place
    return v.x + v.y + v.z + v.w;
    // Should be 11.0 + 22.0 + 33.0 + 44.0 = 110.0
}

// run: test_vec4_add_assign() ~= 110.0

float test_vec4_sub_assign() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 v2 = vec4(1.0, 2.0, 3.0, 4.0);
    v -= v2;
    // Component-wise subtraction in place
    return v.x + v.y + v.z + v.w;
    // Should be 9.0 + 18.0 + 27.0 + 36.0 = 90.0
}

// run: test_vec4_sub_assign() ~= 90.0

float test_vec4_mul_assign() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(5.0, 6.0, 7.0, 8.0);
    v *= v2;
    // Component-wise multiplication in place
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 12.0 + 21.0 + 32.0 = 70.0
}

// run: test_vec4_mul_assign() ~= 70.0

float test_vec4_div_assign() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 v2 = vec4(2.0, 4.0, 5.0, 8.0);
    v /= v2;
    // Component-wise division in place
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 5.0 + 6.0 + 5.0 = 21.0
}

// run: test_vec4_div_assign() ~= 21.0

float test_vec4_add_assign_scalar() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float s = 5.0;
    v += s;
    // Scalar addition in place
    return v.x + v.y + v.z + v.w;
    // Should be 6.0 + 7.0 + 8.0 + 9.0 = 30.0
}

// run: test_vec4_add_assign_scalar() ~= 30.0

float test_vec4_sub_assign_scalar() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    float s = 5.0;
    v -= s;
    // Scalar subtraction in place
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 15.0 + 25.0 + 35.0 = 80.0
}

// run: test_vec4_sub_assign_scalar() ~= 80.0

float test_vec4_mul_assign_scalar() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float s = 5.0;
    v *= s;
    // Scalar multiplication in place
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_vec4_mul_assign_scalar() ~= 50.0

float test_vec4_div_assign_scalar() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    float s = 2.0;
    v /= s;
    // Scalar division in place
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_vec4_div_assign_scalar() ~= 50.0

