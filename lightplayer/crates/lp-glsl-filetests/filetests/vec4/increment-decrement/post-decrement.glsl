// test run
// target riscv32.fixed32

// ============================================================================
// Post-decrement: v-- (returns then decrements)
// ============================================================================

float test_vec4_post_decrement() {
    vec4 v = vec4(3.0, 4.0, 5.0, 6.0);
    vec4 old_v = v--;
    // Should return old value (3.0, 4.0, 5.0, 6.0), then decrement v
    return old_v.x + old_v.y + old_v.z + old_v.w;
    // Should be 3.0 + 4.0 + 5.0 + 6.0 = 18.0
}

// run: test_vec4_post_decrement() ~= 18.0

float test_vec4_post_decrement_verify_value() {
    vec4 v = vec4(3.0, 4.0, 5.0, 6.0);
    v--;
    // Verify v was decremented after post-decrement
    return v.x + v.y + v.z + v.w;
    // Should be 2.0 + 3.0 + 4.0 + 5.0 = 14.0
}

// run: test_vec4_post_decrement_verify_value() ~= 14.0

float test_vec4_post_decrement_verify_return() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = v--;
    // Verify returned value is old value
    float sum = 0.0;
    if (result.x == 10.0) sum = sum + 1.0;
    if (result.y == 20.0) sum = sum + 1.0;
    if (result.z == 30.0) sum = sum + 1.0;
    if (result.w == 40.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components returned correctly)
}

// run: test_vec4_post_decrement_verify_return() ~= 4.0

float test_vec4_post_decrement_negative() {
    vec4 v = vec4(-1.0, -2.0, -3.0, -4.0);
    vec4 old_v = v--;
    // Post-decrement negative values
    return old_v.x + old_v.y + old_v.z + old_v.w;
    // Should be -1.0 + -2.0 + -3.0 + -4.0 = -10.0
}

// run: test_vec4_post_decrement_negative() ~= -10.0

float test_vec4_post_decrement_decimal() {
    vec4 v = vec4(1.5, 2.5, 3.5, 4.5);
    vec4 old_v = v--;
    // Post-decrement decimal values
    return old_v.x + old_v.y + old_v.z + old_v.w;
    // Should be 1.5 + 2.5 + 3.5 + 4.5 = 12.0
}

// run: test_vec4_post_decrement_decimal() ~= 12.0

