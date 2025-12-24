// test run
// target riscv32.fixed32

// ============================================================================
// Nested swizzling: (v.xy).yx, etc.
// ============================================================================

float test_vec4_nested_swizzle_xy_yx() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec2 result = (v.xy).yx;
    // Extract xy, then swap to yx
    return result.x + result.y;
    // Should be 2.0 + 1.0 = 3.0
}

// run: test_vec4_nested_swizzle_xy_yx() ~= 3.0

float test_vec4_nested_swizzle_xyz_zyx() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec3 result = (v.xyz).zyx;
    // Extract xyz, then reverse to zyx
    return result.x + result.y + result.z;
    // Should be 3.0 + 2.0 + 1.0 = 6.0
}

// run: test_vec4_nested_swizzle_xyz_zyx() ~= 6.0

float test_vec4_nested_swizzle_xyzw_wzyx() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = (v.xyzw).wzyx;
    // Extract xyzw, then reverse to wzyx
    return result.x + result.y + result.z + result.w;
    // Should be 4.0 + 3.0 + 2.0 + 1.0 = 10.0
}

// run: test_vec4_nested_swizzle_xyzw_wzyx() ~= 10.0

float test_vec4_nested_swizzle_rgba_abgr() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = (v.rgba).abgr;
    // Extract rgba, then reverse to abgr
    return result.r + result.g + result.b + result.a;
    // Should be 40.0 + 30.0 + 20.0 + 10.0 = 100.0
}

// run: test_vec4_nested_swizzle_rgba_abgr() ~= 100.0

float test_vec4_nested_swizzle_xy_xx() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec2 result = (v.xy).xx;
    // Extract xy, then duplicate x
    return result.x + result.y;
    // Should be 1.0 + 1.0 = 2.0
}

// run: test_vec4_nested_swizzle_xy_xx() ~= 2.0

float test_vec4_nested_swizzle_xyz_yyy() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec3 result = (v.xyz).yyy;
    // Extract xyz, then duplicate y three times
    return result.x + result.y + result.z;
    // Should be 2.0 + 2.0 + 2.0 = 6.0
}

// run: test_vec4_nested_swizzle_xyz_yyy() ~= 6.0

float test_vec4_nested_swizzle_chain() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec2 step1 = v.xy;
    vec2 step2 = step1.yx;
    vec2 step3 = step2.xx;
    // Chain: xy -> yx -> xx
    return step3.x + step3.y;
    // Should be 2.0 + 2.0 = 4.0
}

// run: test_vec4_nested_swizzle_chain() ~= 4.0

float test_vec4_nested_swizzle_verify_order() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec2 result = (v.xy).yx;
    // Verify nested swizzle order
    float sum = 0.0;
    if (result.x == 2.0) sum = sum + 1.0;
    if (result.y == 1.0) sum = sum + 1.0;
    return sum;
    // Should be 2.0 (both components swapped correctly)
}

// run: test_vec4_nested_swizzle_verify_order() ~= 2.0

