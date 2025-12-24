// test run
// target riscv32.fixed32

// ============================================================================
// Access Component: bvec2.x, bvec2.y - component name access
// ============================================================================

bool test_bvec2_access_component_x() {
    // Component name access
    bvec2 a = bvec2(true, false);
    return a.x;
    // Should be true
}

// run: test_bvec2_access_component_x() == true

bool test_bvec2_access_component_y() {
    bvec2 a = bvec2(true, false);
    return a.y;
    // Should be false
}

// run: test_bvec2_access_component_y() == false

bool test_bvec2_access_component_x_from_expression() {
    return not(bvec2(false, true)).x;
    // Should be true (not(bvec2(false, true)).x = bvec2(true, false).x = true)
}

// run: test_bvec2_access_component_x_from_expression() == true

bool test_bvec2_access_component_y_from_expression() {
    return not(bvec2(true, false)).y;
    // Should be true (not(bvec2(true, false)).y = bvec2(false, true).y = true)
}

// run: test_bvec2_access_component_y_from_expression() == true

bool test_bvec2_access_component_in_assignment() {
    bvec2 a = bvec2(true, false);
    bool result = a.x;
    return result;
    // Should be true
}

// run: test_bvec2_access_component_in_assignment() == true

bool test_bvec2_access_component_both() {
    bvec2 a = bvec2(true, false);
    bool result = a.x && !a.y;
    return result;
    // Should be true (true && !false = true && true = true)
}

// run: test_bvec2_access_component_both() == true

bool test_bvec2_access_component_mixed_names() {
    // Test different name sets (xy, rg, st)
    bvec2 a = bvec2(true, false);
    return a.r; // Same as x
    // Should be true
}

// run: test_bvec2_access_component_mixed_names() == true
