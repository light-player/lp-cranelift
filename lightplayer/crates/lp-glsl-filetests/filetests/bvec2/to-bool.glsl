// test run
// target riscv32.fixed32

// ============================================================================
// To Bool: bool(bvec2) - extract first component
// ============================================================================

bool test_bvec2_to_bool_true() {
    // Conversion bool(bvec2) extracts first component
    bvec2 source = bvec2(true, false);
    return bool(source);
    // Should be true
}

// run: test_bvec2_to_bool_true() == true

bool test_bvec2_to_bool_false() {
    bvec2 source = bvec2(false, true);
    return bool(source);
    // Should be false
}

// run: test_bvec2_to_bool_false() == false

bool test_bvec2_to_bool_all_true() {
    bvec2 source = bvec2(true, true);
    return bool(source);
    // Should be true
}

// run: test_bvec2_to_bool_all_true() == true

bool test_bvec2_to_bool_all_false() {
    bvec2 source = bvec2(false, false);
    return bool(source);
    // Should be false
}

// run: test_bvec2_to_bool_all_false() == false

bool test_bvec2_to_bool_variable() {
    bvec2 x = bvec2(true, false);
    return bool(x);
    // Should be true
}

// run: test_bvec2_to_bool_variable() == true

bool test_bvec2_to_bool_expression() {
    return bool(not(bvec2(false, true)));
    // Should be true (bool(not(bvec2(false, true))) = bool(bvec2(true, false)) = true)
}

// run: test_bvec2_to_bool_expression() == true

bool test_bvec2_to_bool_in_condition() {
    bvec2 x = bvec2(true, false);
    if (bool(x)) {
        return true;
    } else {
        return false;
    }
    // Should be true
}

// run: test_bvec2_to_bool_in_condition() == true
