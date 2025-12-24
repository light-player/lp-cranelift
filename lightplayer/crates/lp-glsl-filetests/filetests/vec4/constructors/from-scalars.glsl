// test run
// target riscv32.fixed32

// ============================================================================
// Vector constructor from multiple scalars: vec4(float, float, float, float)
// ============================================================================

float test_vec4_from_scalars() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Components: x=1.0, y=2.0, z=3.0, w=4.0
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_from_scalars() ~= 10.0

float test_vec4_from_scalars_verify_order() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    // Verify component order: x=10.0, y=20.0, z=30.0, w=40.0
    float sum = 0.0;
    if (v.x == 10.0) sum = sum + 1.0;
    if (v.y == 20.0) sum = sum + 1.0;
    if (v.z == 30.0) sum = sum + 1.0;
    if (v.w == 40.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components in correct order)
}

// run: test_vec4_from_scalars_verify_order() ~= 4.0

float test_vec4_from_scalars_negative() {
    vec4 v = vec4(-1.0, -2.0, -3.0, -4.0);
    // All components negative
    return v.x + v.y + v.z + v.w;
    // Should be -1.0 + -2.0 + -3.0 + -4.0 = -10.0
}

// run: test_vec4_from_scalars_negative() ~= -10.0

float test_vec4_from_scalars_mixed() {
    vec4 v = vec4(1.5, -2.5, 3.5, -4.5);
    // Mixed positive and negative
    return v.x + v.y + v.z + v.w;
    // Should be 1.5 + -2.5 + 3.5 + -4.5 = -2.0
}

// run: test_vec4_from_scalars_mixed() ~= -2.0

float test_vec4_from_scalars_decimal() {
    vec4 v = vec4(0.1, 0.2, 0.3, 0.4);
    // Small decimal values
    return v.x + v.y + v.z + v.w;
    // Should be 0.1 + 0.2 + 0.3 + 0.4 = 1.0
}

// run: test_vec4_from_scalars_decimal() ~= 1.0

