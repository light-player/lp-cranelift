// test run
// target riscv32.fixed32

// ============================================================================
// Element assignment: v.x = float, v[0] = float
// ============================================================================

float test_vec4_element_assignment_x() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.x = 100.0;
    // Assign to x component
    return v.x + v.y + v.z + v.w;
    // Should be 100.0 + 2.0 + 3.0 + 4.0 = 109.0
}

// run: test_vec4_element_assignment_x() ~= 109.0

float test_vec4_element_assignment_y() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.y = 200.0;
    // Assign to y component
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 200.0 + 3.0 + 4.0 = 208.0
}

// run: test_vec4_element_assignment_y() ~= 208.0

float test_vec4_element_assignment_z() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.z = 300.0;
    // Assign to z component
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 300.0 + 4.0 = 307.0
}

// run: test_vec4_element_assignment_z() ~= 307.0

float test_vec4_element_assignment_w() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.w = 400.0;
    // Assign to w component
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 400.0 = 406.0
}

// run: test_vec4_element_assignment_w() ~= 406.0

float test_vec4_element_assignment_array_index() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v[0] = 100.0;  // Assign to index 0 (x)
    v[1] = 200.0;  // Assign to index 1 (y)
    v[2] = 300.0;  // Assign to index 2 (z)
    v[3] = 400.0;  // Assign to index 3 (w)
    return v.x + v.y + v.z + v.w;
    // Should be 100.0 + 200.0 + 300.0 + 400.0 = 1000.0
}

// run: test_vec4_element_assignment_array_index() ~= 1000.0

float test_vec4_element_assignment_verify_others() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    v.x = 100.0;  // Modify x
    // Verify other components unchanged
    return v.y + v.z + v.w;
    // Should be 20.0 + 30.0 + 40.0 = 90.0
}

// run: test_vec4_element_assignment_verify_others() ~= 90.0

float test_vec4_element_assignment_all_components() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.x = 10.0;
    v.y = 20.0;
    v.z = 30.0;
    v.w = 40.0;
    // Assign all components
    float sum = 0.0;
    if (v.x == 10.0) sum = sum + 1.0;
    if (v.y == 20.0) sum = sum + 1.0;
    if (v.z == 30.0) sum = sum + 1.0;
    if (v.w == 40.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components assigned correctly)
}

// run: test_vec4_element_assignment_all_components() ~= 4.0

