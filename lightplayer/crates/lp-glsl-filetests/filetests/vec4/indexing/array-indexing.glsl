// test run
// target riscv32.fixed32

// ============================================================================
// Array indexing: v[0], v[1], v[2], v[3]
// ============================================================================

float test_vec4_array_indexing() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Access using array indexing
    return v[0] + v[1] + v[2] + v[3];
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_array_indexing() ~= 10.0

float test_vec4_array_indexing_verify_order() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    // Verify index order: [0]=x, [1]=y, [2]=z, [3]=w
    float sum = 0.0;
    if (v[0] == 10.0) sum = sum + 1.0;
    if (v[1] == 20.0) sum = sum + 1.0;
    if (v[2] == 30.0) sum = sum + 1.0;
    if (v[3] == 40.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all indices correct)
}

// run: test_vec4_array_indexing_verify_order() ~= 4.0

float test_vec4_array_indexing_equals_component() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Verify array indexing equals component access
    float sum = 0.0;
    if (v[0] == v.x) sum = sum + 1.0;
    if (v[1] == v.y) sum = sum + 1.0;
    if (v[2] == v.z) sum = sum + 1.0;
    if (v[3] == v.w) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all indices match component names)
}

// run: test_vec4_array_indexing_equals_component() ~= 4.0

float test_vec4_array_indexing_read() {
    vec4 v = vec4(100.0, 200.0, 300.0, 400.0);
    // Read using array indices
    float val0 = v[0];
    float val1 = v[1];
    float val2 = v[2];
    float val3 = v[3];
    return val0 + val1 + val2 + val3;
    // Should be 100.0 + 200.0 + 300.0 + 400.0 = 1000.0
}

// run: test_vec4_array_indexing_read() ~= 1000.0

float test_vec4_array_indexing_variable() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Use variable index (if supported)
    int idx = 0;
    float sum = v[idx];
    idx = 1;
    sum = sum + v[idx];
    idx = 2;
    sum = sum + v[idx];
    idx = 3;
    sum = sum + v[idx];
    return sum;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_array_indexing_variable() ~= 10.0

