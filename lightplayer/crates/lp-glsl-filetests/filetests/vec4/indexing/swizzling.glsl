// test run
// target riscv32.fixed32

// ============================================================================
// Swizzling: v.xy, v.xyz, v.xyzw, v.rgba, v.stpq, v.xxyy, v.wzyx
// ============================================================================

float test_vec4_swizzle_xy() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec2 result = v.xy;
    // Extract first two components
    return result.x + result.y;
    // Should be 1.0 + 2.0 = 3.0
}

// run: test_vec4_swizzle_xy() ~= 3.0

float test_vec4_swizzle_xyz() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec3 result = v.xyz;
    // Extract first three components
    return result.x + result.y + result.z;
    // Should be 1.0 + 2.0 + 3.0 = 6.0
}

// run: test_vec4_swizzle_xyz() ~= 6.0

float test_vec4_swizzle_xyzw() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v.xyzw;
    // Extract all components (identity)
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_swizzle_xyzw() ~= 10.0

float test_vec4_swizzle_rgba() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = v.rgba;
    // Extract using rgba swizzle
    return result.r + result.g + result.b + result.a;
    // Should be 10.0 + 20.0 + 30.0 + 40.0 = 100.0
}

// run: test_vec4_swizzle_rgba() ~= 100.0

float test_vec4_swizzle_stpq() {
    vec4 v = vec4(100.0, 200.0, 300.0, 400.0);
    vec4 result = v.stpq;
    // Extract using stpq swizzle
    return result.s + result.t + result.p + result.q;
    // Should be 100.0 + 200.0 + 300.0 + 400.0 = 1000.0
}

// run: test_vec4_swizzle_stpq() ~= 1000.0

float test_vec4_swizzle_reverse() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v.wzyx;
    // Reverse order swizzle
    return result.x + result.y + result.z + result.w;
    // Should be 4.0 + 3.0 + 2.0 + 1.0 = 10.0
}

// run: test_vec4_swizzle_reverse() ~= 10.0

float test_vec4_swizzle_duplicate() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v.xxyy;
    // Duplicate components
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 1.0 + 2.0 + 2.0 = 6.0
}

// run: test_vec4_swizzle_duplicate() ~= 6.0

float test_vec4_swizzle_mixed() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v.xzyw;
    // Mixed order swizzle
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 3.0 + 2.0 + 4.0 = 10.0
}

// run: test_vec4_swizzle_mixed() ~= 10.0

float test_vec4_swizzle_partial() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec2 result = v.zw;
    // Extract z and w components
    return result.x + result.y;
    // Should be 30.0 + 40.0 = 70.0
}

// run: test_vec4_swizzle_partial() ~= 70.0

float test_vec4_swizzle_verify_order() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v.wzyx;
    // Verify reverse order
    float sum = 0.0;
    if (result.x == 4.0) sum = sum + 1.0;
    if (result.y == 3.0) sum = sum + 1.0;
    if (result.z == 2.0) sum = sum + 1.0;
    if (result.w == 1.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components in reverse order)
}

// run: test_vec4_swizzle_verify_order() ~= 4.0

