// test run
// target riscv32.fixed32

// ============================================================================
// Greater than: greaterThan(vec4, vec4) -> bvec4 (component-wise)
// ============================================================================

bool test_vec4_greater_than() {
    vec4 a = vec4(5.0, 6.0, 7.0, 8.0);
    vec4 b = vec4(1.0, 2.0, 3.0, 4.0);
    bvec4 result = greaterThan(a, b);
    // All components of a are greater than b
    // result should be (true, true, true, true)
    return all(result);
    // Should be true (all components greater)
}

// run: test_vec4_greater_than() == true

bool test_vec4_greater_than_mixed() {
    vec4 a = vec4(5.0, 1.0, 7.0, 8.0);
    vec4 b = vec4(2.0, 3.0, 4.0, 5.0);
    bvec4 result = greaterThan(a, b);
    // Mixed: a.x > b.x (true), a.y < b.y (false), a.z > b.z (true), a.w > b.w (true)
    // result should be (true, false, true, true)
    return any(result);
    // Should be true (at least one component greater)
}

// run: test_vec4_greater_than_mixed() == true

bool test_vec4_greater_than_none() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(5.0, 6.0, 7.0, 8.0);
    bvec4 result = greaterThan(a, b);
    // No components of a are greater than b
    // result should be (false, false, false, false)
    return any(result);
    // Should be false (no components greater)
}

// run: test_vec4_greater_than_none() == false

bool test_vec4_greater_than_equal() {
    vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 b = vec4(1.0, 2.0, 3.0, 4.0);
    bvec4 result = greaterThan(a, b);
    // Equal vectors - none greater than
    // result should be (false, false, false, false)
    return any(result);
    // Should be false (equal, not greater)
}

// run: test_vec4_greater_than_equal() == false

