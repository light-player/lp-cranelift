// test run
// target riscv32.fixed32

// ============================================================================
// To Int: int(bvec2) - converts first component (false -> 0, true -> 1)
// ============================================================================

int test_bvec2_to_int_true() {
    // Conversion int(bvec2) converts first component (false -> 0, true -> 1)
    bvec2 source = bvec2(true, false);
    return int(source);
    // Should be 1
}

// run: test_bvec2_to_int_true() == 1

int test_bvec2_to_int_false() {
    bvec2 source = bvec2(false, true);
    return int(source);
    // Should be 0
}

// run: test_bvec2_to_int_false() == 0

int test_bvec2_to_int_all_true() {
    bvec2 source = bvec2(true, true);
    return int(source);
    // Should be 1
}

// run: test_bvec2_to_int_all_true() == 1

int test_bvec2_to_int_all_false() {
    bvec2 source = bvec2(false, false);
    return int(source);
    // Should be 0
}

// run: test_bvec2_to_int_all_false() == 0

int test_bvec2_to_int_variable() {
    bvec2 x = bvec2(true, false);
    return int(x);
    // Should be 1
}

// run: test_bvec2_to_int_variable() == 1

int test_bvec2_to_int_expression() {
    return int(not(bvec2(false, true)));
    // Should be 1 (int(not(bvec2(false, true))) = int(bvec2(true, false)) = 1)
}

// run: test_bvec2_to_int_expression() == 1

int test_bvec2_to_int_in_arithmetic() {
    bvec2 x = bvec2(true, false);
    return int(x) + 5;
    // Should be 6 (1 + 5)
}

// run: test_bvec2_to_int_in_arithmetic() == 6
