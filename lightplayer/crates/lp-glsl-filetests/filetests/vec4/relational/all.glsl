// test run
// target riscv32.fixed32

// ============================================================================
// All: all(bvec4) -> bool - returns true if all components are true
// ============================================================================

bool test_vec4_all_all_true() {
    bvec4 v = bvec4(true, true, true, true);
    return all(v);
    // Should be true (all components true)
}

// run: test_vec4_all_all_true() == true

bool test_vec4_all_some_false() {
    bvec4 v = bvec4(true, false, true, true);
    return all(v);
    // Should be false (not all components true)
}

// run: test_vec4_all_some_false() == false

bool test_vec4_all_all_false() {
    bvec4 v = bvec4(false, false, false, false);
    return all(v);
    // Should be false (no components true)
}

// run: test_vec4_all_all_false() == false

bool test_vec4_all_one_false() {
    bvec4 v = bvec4(true, true, false, true);
    return all(v);
    // Should be false (one component false)
}

// run: test_vec4_all_one_false() == false

bool test_vec4_all_mixed() {
    bvec4 v = bvec4(true, false, true, false);
    return all(v);
    // Should be false (not all components true)
}

// run: test_vec4_all_mixed() == false

