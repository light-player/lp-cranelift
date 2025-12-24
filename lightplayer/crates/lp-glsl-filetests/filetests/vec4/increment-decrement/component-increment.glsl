// test run
// target riscv32.fixed32

// ============================================================================
// Component-level increment/decrement: v.x++, ++v.y, etc.
// ============================================================================

float test_vec4_component_pre_increment() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float result = ++v.x;
    // Pre-increment x component
    return result + v.y + v.z + v.w;
    // Should be 2.0 + 2.0 + 3.0 + 4.0 = 11.0
}

// run: test_vec4_component_pre_increment() ~= 11.0

float test_vec4_component_post_increment() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float old_x = v.x++;
    // Post-increment x component
    return old_x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_component_post_increment() ~= 10.0

float test_vec4_component_pre_decrement() {
    vec4 v = vec4(3.0, 4.0, 5.0, 6.0);
    float result = --v.y;
    // Pre-decrement y component
    return v.x + result + v.z + v.w;
    // Should be 3.0 + 3.0 + 5.0 + 6.0 = 17.0
}

// run: test_vec4_component_pre_decrement() ~= 17.0

float test_vec4_component_post_decrement() {
    vec4 v = vec4(3.0, 4.0, 5.0, 6.0);
    float old_z = v.z--;
    // Post-decrement z component
    return v.x + v.y + old_z + v.w;
    // Should be 3.0 + 4.0 + 5.0 + 6.0 = 18.0
}

// run: test_vec4_component_post_decrement() ~= 18.0

float test_vec4_component_multiple_increment() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    ++v.x;
    ++v.y;
    ++v.z;
    ++v.w;
    // Increment all components
    return v.x + v.y + v.z + v.w;
    // Should be 2.0 + 3.0 + 4.0 + 5.0 = 14.0
}

// run: test_vec4_component_multiple_increment() ~= 14.0

float test_vec4_component_verify_others_unchanged() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    ++v.x;
    // Verify other components unchanged
    return v.y + v.z + v.w;
    // Should be 20.0 + 30.0 + 40.0 = 90.0
}

// run: test_vec4_component_verify_others_unchanged() ~= 90.0

