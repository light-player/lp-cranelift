// test run
// target riscv32.fixed32

// ============================================================================
// Vector division: vec4 / vec4, vec4 / float (component-wise)
// ============================================================================

float test_vec4_division() {
    vec4 v1 = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 v2 = vec4(2.0, 4.0, 5.0, 8.0);
    vec4 result = v1 / v2;
    // Component-wise division
    return result.x + result.y + result.z + result.w;
    // Should be 5.0 + 5.0 + 6.0 + 5.0 = 21.0
}

// run: test_vec4_division() ~= 21.0

float test_vec4_division_scalar() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    float s = 2.0;
    vec4 result = v / s;
    // Scalar division (component-wise)
    return result.x + result.y + result.z + result.w;
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_vec4_division_scalar() ~= 50.0

float test_vec4_division_verify_components() {
    vec4 v1 = vec4(12.0, 18.0, 24.0, 30.0);
    vec4 v2 = vec4(2.0, 3.0, 4.0, 5.0);
    vec4 result = v1 / v2;
    // Verify each component
    float sum = 0.0;
    if (result.x == 6.0) sum = sum + 1.0;
    if (result.y == 6.0) sum = sum + 1.0;
    if (result.z == 6.0) sum = sum + 1.0;
    if (result.w == 6.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components correct)
}

// run: test_vec4_division_verify_components() ~= 4.0

float test_vec4_division_decimal() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(2.0, 2.0, 2.0, 2.0);
    vec4 result = v1 / v2;
    // Division resulting in decimals
    return result.x + result.y + result.z + result.w;
    // Should be 0.5 + 1.0 + 1.5 + 2.0 = 5.0
}

// run: test_vec4_division_decimal() ~= 5.0

float test_vec4_division_scalar_decimal() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float s = 4.0;
    vec4 result = v / s;
    // Division by scalar
    return result.x + result.y + result.z + result.w;
    // Should be 0.25 + 0.5 + 0.75 + 1.0 = 2.5
}

// run: test_vec4_division_scalar_decimal() ~= 2.5

