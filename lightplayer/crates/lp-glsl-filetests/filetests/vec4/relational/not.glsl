// test run
// target riscv32.fixed32

// ============================================================================
// Not: not(bvec4) -> bvec4 - component-wise logical NOT
// ============================================================================

bvec4 test_vec4_not_all_true() {
    bvec4 v = bvec4(true, true, true, true);
    return not(v);
    // Should be (false, false, false, false)
}

// run: test_vec4_not_all_true() == bvec4(false, false, false, false)

bvec4 test_vec4_not_all_false() {
    bvec4 v = bvec4(false, false, false, false);
    return not(v);
    // Should be (true, true, true, true)
}

// run: test_vec4_not_all_false() == bvec4(true, true, true, true)

bvec4 test_vec4_not_mixed() {
    bvec4 v = bvec4(true, false, true, false);
    return not(v);
    // Should be (false, true, false, true)
}

// run: test_vec4_not_mixed() == bvec4(false, true, false, true)

bvec4 test_vec4_not_double_negation() {
    bvec4 v = bvec4(true, false, true, false);
    bvec4 result = not(not(v));
    // Double negation should equal original
    return result;
    // Should be (true, false, true, false)
}

// run: test_vec4_not_double_negation() == bvec4(true, false, true, false)

bvec4 test_vec4_not_verify_components() {
    bvec4 v = bvec4(true, false, true, false);
    bvec4 result = not(v);
    // Verify each component negated
    float sum = 0.0;
    if (result.x == false) sum = sum + 1.0;
    if (result.y == true) sum = sum + 1.0;
    if (result.z == false) sum = sum + 1.0;
    if (result.w == true) sum = sum + 1.0;
    // Convert to bool for return (sum == 4.0 means all correct)
    if (sum == 4.0) {
        return true;
    }
    return false;
    // Should be true (all components negated correctly)
}

// run: test_vec4_not_verify_components() == true

