// test run
// target riscv32.fixed32

// ============================================================================
// Vector shortening: vec2(vec4), vec3(vec4) - dropping components
// ============================================================================

float test_vec2_from_vec4() {
    vec4 v4 = vec4(1.0, 2.0, 3.0, 4.0);
    vec2 v2 = vec2(v4);
    // Should drop z and w, keeping x and y
    return v2.x + v2.y;
    // Should be 1.0 + 2.0 = 3.0
}

// run: test_vec2_from_vec4() ~= 3.0

float test_vec3_from_vec4() {
    vec4 v4 = vec4(1.0, 2.0, 3.0, 4.0);
    vec3 v3 = vec3(v4);
    // Should drop w, keeping x, y, z
    return v3.x + v3.y + v3.z;
    // Should be 1.0 + 2.0 + 3.0 = 6.0
}

// run: test_vec3_from_vec4() ~= 6.0

float test_vec2_from_vec4_verify_order() {
    vec4 v4 = vec4(10.0, 20.0, 30.0, 40.0);
    vec2 v2 = vec2(v4);
    // Verify first two components are preserved
    float sum = 0.0;
    if (v2.x == 10.0) sum = sum + 1.0;
    if (v2.y == 20.0) sum = sum + 1.0;
    return sum;
    // Should be 2.0 (both components match)
}

// run: test_vec2_from_vec4_verify_order() ~= 2.0

float test_vec3_from_vec4_verify_order() {
    vec4 v4 = vec4(100.0, 200.0, 300.0, 400.0);
    vec3 v3 = vec3(v4);
    // Verify first three components are preserved
    float sum = 0.0;
    if (v3.x == 100.0) sum = sum + 1.0;
    if (v3.y == 200.0) sum = sum + 1.0;
    if (v3.z == 300.0) sum = sum + 1.0;
    return sum;
    // Should be 3.0 (all three components match)
}

// run: test_vec3_from_vec4_verify_order() ~= 3.0

float test_shortening_chain() {
    vec4 v4 = vec4(1.0, 2.0, 3.0, 4.0);
    vec3 v3 = vec3(v4);
    vec2 v2 = vec2(v3);
    // Chain: vec4 -> vec3 -> vec2
    return v2.x + v2.y;
    // Should be 1.0 + 2.0 = 3.0
}

// run: test_shortening_chain() ~= 3.0

