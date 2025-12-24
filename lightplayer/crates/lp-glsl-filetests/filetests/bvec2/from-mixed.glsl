// test run
// target riscv32.fixed32

// ============================================================================
// From Mixed: bvec2(int, float) - type conversions (0/0.0 -> false, non-zero -> true)
// ============================================================================

bvec2 test_bvec2_from_mixed_int_int() {
    // Constructor bvec2(int, float) converts to bool (0/0.0 -> false, non-zero -> true)
    return bvec2(0, 1.0);
    // Should be bvec2(false, true)
}

// run: test_bvec2_from_mixed_int_int() == bvec2(false, true)

bvec2 test_bvec2_from_mixed_int_float() {
    return bvec2(1, 0.0);
    // Should be bvec2(true, false)
}

// run: test_bvec2_from_mixed_int_float() == bvec2(true, false)

bvec2 test_bvec2_from_mixed_negative_int() {
    return bvec2(-1, 2);
    // Should be bvec2(true, true) (both non-zero)
}

// run: test_bvec2_from_mixed_negative_int() == bvec2(true, true)

bvec2 test_bvec2_from_mixed_negative_float() {
    return bvec2(0, -1.5);
    // Should be bvec2(false, true) (zero int -> false, non-zero float -> true)
}

// run: test_bvec2_from_mixed_negative_float() == bvec2(false, true)

bvec2 test_bvec2_from_mixed_large_values() {
    return bvec2(100, 0.001);
    // Should be bvec2(true, true) (both non-zero)
}

// run: test_bvec2_from_mixed_large_values() == bvec2(true, true)

bvec2 test_bvec2_from_mixed_variables() {
    int x = 5;
    float y = 0.0;
    return bvec2(x, y);
    // Should be bvec2(true, false)
}

// run: test_bvec2_from_mixed_variables() == bvec2(true, false)

bvec2 test_bvec2_from_mixed_expressions() {
    return bvec2(1 + 2, 3.0 * 0.0);
    // Should be bvec2(true, false) (3 != 0, 0.0 == 0)
}

// run: test_bvec2_from_mixed_expressions() == bvec2(true, false)
