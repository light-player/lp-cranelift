// test run
// target riscv32.fixed32

// ============================================================================
// Swizzle assignment: v.xy = vec2, v.xyz = vec3 (no duplicates)
// ============================================================================

float test_vec4_swizzle_assignment_xy() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xy = vec2(10.0, 20.0);
    // Assign to x and y components
    return v.x + v.y + v.z + v.w;
    // Should be 10.0 + 20.0 + 3.0 + 4.0 = 37.0
}

// run: test_vec4_swizzle_assignment_xy() ~= 37.0

float test_vec4_swizzle_assignment_xyz() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xyz = vec3(10.0, 20.0, 30.0);
    // Assign to x, y, z components
    return v.x + v.y + v.z + v.w;
    // Should be 10.0 + 20.0 + 30.0 + 4.0 = 64.0
}

// run: test_vec4_swizzle_assignment_xyz() ~= 64.0

float test_vec4_swizzle_assignment_xz() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xz = vec2(100.0, 300.0);
    // Assign to x and z components (skipping y)
    return v.x + v.y + v.z + v.w;
    // Should be 100.0 + 2.0 + 300.0 + 4.0 = 406.0
}

// run: test_vec4_swizzle_assignment_xz() ~= 406.0

float test_vec4_swizzle_assignment_yw() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.yw = vec2(200.0, 400.0);
    // Assign to y and w components
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 200.0 + 3.0 + 400.0 = 604.0
}

// run: test_vec4_swizzle_assignment_yw() ~= 604.0

float test_vec4_swizzle_assignment_rgba() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.rgba = vec4(10.0, 20.0, 30.0, 40.0);
    // Assign using rgba swizzle
    return v.x + v.y + v.z + v.w;
    // Should be 10.0 + 20.0 + 30.0 + 40.0 = 100.0
}

// run: test_vec4_swizzle_assignment_rgba() ~= 100.0

float test_vec4_swizzle_assignment_stpq() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.stpq = vec4(100.0, 200.0, 300.0, 400.0);
    // Assign using stpq swizzle
    return v.x + v.y + v.z + v.w;
    // Should be 100.0 + 200.0 + 300.0 + 400.0 = 1000.0
}

// run: test_vec4_swizzle_assignment_stpq() ~= 1000.0

float test_vec4_swizzle_assignment_reverse() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.wzyx = vec4(10.0, 20.0, 30.0, 40.0);
    // Assign in reverse order (wzyx)
    return v.x + v.y + v.z + v.w;
    // Should be 40.0 + 30.0 + 20.0 + 10.0 = 100.0
}

// run: test_vec4_swizzle_assignment_reverse() ~= 100.0

float test_vec4_swizzle_assignment_verify_order() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xy = vec2(100.0, 200.0);
    // Verify assignment order
    float sum = 0.0;
    if (v.x == 100.0) sum = sum + 1.0;
    if (v.y == 200.0) sum = sum + 1.0;
    if (v.z == 3.0) sum = sum + 1.0;
    if (v.w == 4.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components correct)
}

// run: test_vec4_swizzle_assignment_verify_order() ~= 4.0

