// test run
// target riscv32.fixed32

// ============================================================================
// Not: not(bvec2) -> bvec2 (component-wise logical NOT)
// ============================================================================

bvec2 test_bvec2_not_all_true() {
    bvec2 a = bvec2(true, true);
    // Function not() returns bvec2 (component-wise logical NOT)
    return not(a);
    // Should be bvec2(false, false)
}

// run: test_bvec2_not_all_true() == bvec2(false, false)

bvec2 test_bvec2_not_all_false() {
    bvec2 a = bvec2(false, false);
    return not(a);
    // Should be bvec2(true, true)
}

// run: test_bvec2_not_all_false() == bvec2(true, true)

bvec2 test_bvec2_not_mixed() {
    bvec2 a = bvec2(true, false);
    return not(a);
    // Should be bvec2(false, true)
}

// run: test_bvec2_not_mixed() == bvec2(false, true)

bvec2 test_bvec2_not_other_mixed() {
    bvec2 a = bvec2(false, true);
    return not(a);
    // Should be bvec2(true, false)
}

// run: test_bvec2_not_other_mixed() == bvec2(true, false)

bvec2 test_bvec2_not_double_negation() {
    bvec2 a = bvec2(true, false);
    // Double negation should equal original
    return not(not(a));
    // Should be bvec2(true, false)
}

// run: test_bvec2_not_double_negation() == bvec2(true, false)

bvec2 test_bvec2_not_in_expression() {
    bvec2 a = bvec2(true, false);
    bvec2 b = bvec2(false, true);
    return not(a) == b;
    // Should be bvec2(true, true) (not(true,false) == (false,true) -> (false,true) == (false,true) -> (true,true))
}

// run: test_bvec2_not_in_expression() == bvec2(true, true)

bvec2 test_bvec2_not_after_assignment() {
    bvec2 a = bvec2(true, false);
    bvec2 b = not(a);
    b = not(b);
    return b;
    // Should be bvec2(true, false)
}

// run: test_bvec2_not_after_assignment() == bvec2(true, false)
