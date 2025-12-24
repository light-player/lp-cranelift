// test run
// target riscv32.fixed32

// ============================================================================
// Equal: equal(vec4, vec4) -> bvec4, vec4 == vec4 -> bool
// ============================================================================

bool test_vec4_equal_operator() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(1.0, 2.0, 3.0, 4.0);
    // Operator == returns bool (aggregate comparison)
    return a == b;
    // Should be true (all components equal)
}

// run: test_vec4_equal_operator() == true

bool test_vec4_equal_operator_false() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(5.0, 6.0, 7.0, 8.0);
    return a == b;
    // Should be false (components differ)
}

// run: test_vec4_equal_operator_false() == false

bvec4 test_vec4_equal_function() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(1.0, 5.0, 3.0, 6.0);
    // Function equal() returns bvec4 (component-wise)
    return equal(a, b);
    // Should be (true, false, true, false)
}

// run: test_vec4_equal_function() == bvec4(true, false, true, false)

bvec4 test_vec4_equal_function_all_true() {
    vec4 a = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 b = vec4(10.0, 20.0, 30.0, 40.0);
    return equal(a, b);
    // Should be (true, true, true, true)
}

// run: test_vec4_equal_function_all_true() == bvec4(true, true, true, true)

bvec4 test_vec4_equal_function_all_false() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(5.0, 6.0, 7.0, 8.0);
    return equal(a, b);
    // Should be (false, false, false, false)
}

// run: test_vec4_equal_function_all_false() == bvec4(false, false, false, false)

bool test_vec4_equal_operator_partial() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(1.0, 2.0, 5.0, 4.0);
    // Operator == requires all components equal
    return a == b;
    // Should be false (one component differs)
}

// run: test_vec4_equal_operator_partial() == false

