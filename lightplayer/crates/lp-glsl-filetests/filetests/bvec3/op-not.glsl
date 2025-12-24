// test run
// target riscv32.fixed32

// ============================================================================
// Not: not(bvec3) -> bvec3 (component-wise logical NOT)
// ============================================================================

bvec3 test_bvec3_not_all_true() {
    bvec3 a = bvec3(true, true, true);
    // Function not() returns bvec3 (component-wise logical NOT)
    return not(a);
    // Should be bvec3(false, false, false)
}

// run: test_bvec3_not_all_true() == bvec3(false, false, false)

bvec3 test_bvec3_not_all_false() {
    bvec3 a = bvec3(false, false, false);
    return not(a);
    // Should be bvec3(true, true, true)
}

// run: test_bvec3_not_all_false() == bvec3(true, true, true)

bvec3 test_bvec3_not_mixed() {
    bvec3 a = bvec3(true, false, true);
    return not(a);
    // Should be bvec3(false, true, false)
}

// run: test_bvec3_not_mixed() == bvec3(false, true, false)

bvec3 test_bvec3_not_other_mixed() {
    bvec3 a = bvec3(false, true, false);
    return not(a);
    // Should be bvec3(true, false, true)
}

// run: test_bvec3_not_other_mixed() == bvec3(true, false, true)

bvec3 test_bvec3_not_double_negation() {
    bvec3 a = bvec3(true, false, true);
    // Double negation should equal original
    return not(not(a));
    // Should be bvec3(true, false, true)
}

// run: test_bvec3_not_double_negation() == bvec3(true, false, true)

bvec3 test_bvec3_not_in_expression() {
    bvec3 a = bvec3(true, false, true);
    bvec3 b = bvec3(false, true, false);
    return not(a) == b;
    // Should be bvec3(true, true, true) (not(true,false,true) == (false,true,false) -> (false,true,false) == (false,true,false) -> (true,true,true))
}

// run: test_bvec3_not_in_expression() == bvec3(true, true, true)

bvec3 test_bvec3_not_after_assignment() {
    bvec3 a = bvec3(true, false, true);
    bvec3 b = not(a);
    b = not(b);
    return b;
    // Should be bvec3(true, false, true)
}

// run: test_bvec3_not_after_assignment() == bvec3(true, false, true)
