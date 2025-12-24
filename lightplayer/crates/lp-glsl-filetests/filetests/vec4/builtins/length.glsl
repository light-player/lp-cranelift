// test run
// target riscv32.fixed32

// ============================================================================
// Vector length: v.length() - returns number of components (4 for vec4)
// ============================================================================

int test_vec4_length_method() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    return v.length();
    // Should return 4 (number of components)
}

// run: test_vec4_length_method() == 4

int test_vec4_length_method_any_vector() {
    vec4 v = vec4(100.0, 200.0, 300.0, 400.0);
    return v.length();
    // Should return 4 regardless of values
}

// run: test_vec4_length_method_any_vector() == 4

int test_vec4_length_method_zero() {
    vec4 v = vec4(0.0, 0.0, 0.0, 0.0);
    return v.length();
    // Should return 4 even for zero vector
}

// run: test_vec4_length_method_zero() == 4

