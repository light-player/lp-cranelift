// test run
// target riscv32.fixed32

// ============================================================================
// Vector constructor from smaller vectors
// vec4(vec2, vec2), vec4(vec3, float), vec4(float, vec3), etc.
// ============================================================================

float test_vec4_from_vec2_vec2() {
    vec4 v = vec4(vec2(1.0, 2.0), vec2(3.0, 4.0));
    // Components: x=1.0, y=2.0, z=3.0, w=4.0
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_from_vec2_vec2() ~= 10.0

float test_vec4_from_vec3_float() {
    vec4 v = vec4(vec3(1.0, 2.0, 3.0), 4.0);
    // Components: x=1.0, y=2.0, z=3.0, w=4.0
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_from_vec3_float() ~= 10.0

float test_vec4_from_float_vec3() {
    vec4 v = vec4(1.0, vec3(2.0, 3.0, 4.0));
    // Components: x=1.0, y=2.0, z=3.0, w=4.0
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_from_float_vec3() ~= 10.0

float test_vec4_from_vec2_vec2_verify_order() {
    vec4 v = vec4(vec2(10.0, 20.0), vec2(30.0, 40.0));
    // Verify order: first vec2 fills x,y, second vec2 fills z,w
    float sum = 0.0;
    if (v.x == 10.0) sum = sum + 1.0;
    if (v.y == 20.0) sum = sum + 1.0;
    if (v.z == 30.0) sum = sum + 1.0;
    if (v.w == 40.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components in correct order)
}

// run: test_vec4_from_vec2_vec2_verify_order() ~= 4.0

float test_vec4_from_vec3_float_verify_order() {
    vec4 v = vec4(vec3(5.0, 10.0, 15.0), 20.0);
    // Verify order: vec3 fills x,y,z, float fills w
    float sum = 0.0;
    if (v.x == 5.0) sum = sum + 1.0;
    if (v.y == 10.0) sum = sum + 1.0;
    if (v.z == 15.0) sum = sum + 1.0;
    if (v.w == 20.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components in correct order)
}

// run: test_vec4_from_vec3_float_verify_order() ~= 4.0

float test_vec4_from_float_vec3_verify_order() {
    vec4 v = vec4(100.0, vec3(200.0, 300.0, 400.0));
    // Verify order: float fills x, vec3 fills y,z,w
    float sum = 0.0;
    if (v.x == 100.0) sum = sum + 1.0;
    if (v.y == 200.0) sum = sum + 1.0;
    if (v.z == 300.0) sum = sum + 1.0;
    if (v.w == 400.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components in correct order)
}

// run: test_vec4_from_float_vec3_verify_order() ~= 4.0

