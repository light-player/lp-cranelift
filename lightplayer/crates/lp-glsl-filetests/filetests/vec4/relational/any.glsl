// test run
// target riscv32.fixed32

// ============================================================================
// Any: any(bvec4) -> bool - returns true if any component is true
// ============================================================================

bool test_vec4_any_all_true() {
    bvec4 v = bvec4(true, true, true, true);
    return any(v);
    // Should be true (all components true)
}

// run: test_vec4_any_all_true() == true

bool test_vec4_any_some_true() {
    bvec4 v = bvec4(true, false, false, false);
    return any(v);
    // Should be true (at least one component true)
}

// run: test_vec4_any_some_true() == true

bool test_vec4_any_all_false() {
    bvec4 v = bvec4(false, false, false, false);
    return any(v);
    // Should be false (no components true)
}

// run: test_vec4_any_all_false() == false

bool test_vec4_any_mixed() {
    bvec4 v = bvec4(false, true, false, true);
    return any(v);
    // Should be true (some components true)
}

// run: test_vec4_any_mixed() == true

bool test_vec4_any_one_true() {
    bvec4 v = bvec4(false, false, true, false);
    return any(v);
    // Should be true (one component true)
}

// run: test_vec4_any_one_true() == true

