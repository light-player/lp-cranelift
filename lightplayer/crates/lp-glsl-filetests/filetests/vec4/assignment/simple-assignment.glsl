// test run
// target riscv32.fixed32

// ============================================================================
// Simple vector assignment: vec4 = vec4
// ============================================================================

float test_vec4_assignment() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(10.0, 20.0, 30.0, 40.0);
    v1 = v2;
    // After assignment, v1 should equal v2
    return v1.x + v1.y + v1.z + v1.w;
    // Should be 10.0 + 20.0 + 30.0 + 40.0 = 100.0
}

// run: test_vec4_assignment() ~= 100.0

float test_vec4_assignment_independence() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(10.0, 20.0, 30.0, 40.0);
    v1 = v2;
    v2.x = 100.0;  // Modify v2
    // v1 should be unchanged (independent copy)
    return v1.x;
    // Should be 10.0 (not 100.0)
}

// run: test_vec4_assignment_independence() ~= 10.0

float test_vec4_assignment_verify_components() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(100.0, 200.0, 300.0, 400.0);
    v1 = v2;
    // Verify all components
    float sum = 0.0;
    if (v1.x == 100.0) sum = sum + 1.0;
    if (v1.y == 200.0) sum = sum + 1.0;
    if (v1.z == 300.0) sum = sum + 1.0;
    if (v1.w == 400.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components match)
}

// run: test_vec4_assignment_verify_components() ~= 4.0

float test_vec4_assignment_self() {
    vec4 v = vec4(5.0, 10.0, 15.0, 20.0);
    v = v;  // Self-assignment
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_vec4_assignment_self() ~= 50.0

