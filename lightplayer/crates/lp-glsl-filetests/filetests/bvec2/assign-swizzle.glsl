// test run
// target riscv32.fixed32

// ============================================================================
// Assign Swizzle: bvec2.xy = bvec2 - multi-component swizzle assignment
// ============================================================================

bvec2 test_bvec2_assign_swizzle_xy_full() {
    // Assign to full swizzle
    bvec2 a = bvec2(false, false);
    bvec2 source = bvec2(true, true);
    a.xy = source;
    return a;
    // Should be bvec2(true, true)
}

// run: test_bvec2_assign_swizzle_xy_full() == bvec2(true, true)

bvec2 test_bvec2_assign_swizzle_xy_partial() {
    bvec2 a = bvec2(true, false);
    bvec2 source = bvec2(false, true);
    a.xy = source;
    return a;
    // Should be bvec2(false, true)
}

// run: test_bvec2_assign_swizzle_xy_partial() == bvec2(false, true)

bvec2 test_bvec2_assign_swizzle_yx() {
    // Reverse swizzle assignment
    bvec2 a = bvec2(false, false);
    bvec2 source = bvec2(true, false);
    a.yx = source;
    return a;
    // Should be bvec2(false, true) (source.yx = (false, true) assigned to a.yx)
}

// run: test_bvec2_assign_swizzle_yx() == bvec2(false, true)

bvec2 test_bvec2_assign_swizzle_xx() {
    // Duplicate swizzle assignment
    bvec2 a = bvec2(false, false);
    bvec2 source = bvec2(true, false);
    a.xx = source;
    return a;
    // Should be bvec2(true, true) (source.xx = (true, true) assigned to a.xx)
}

// run: test_bvec2_assign_swizzle_xx() == bvec2(true, true)

bvec2 test_bvec2_assign_swizzle_yy() {
    bvec2 a = bvec2(false, false);
    bvec2 source = bvec2(true, false);
    a.yy = source;
    return a;
    // Should be bvec2(false, false) (source.yy = (false, false) assigned to a.yy)
}

// run: test_bvec2_assign_swizzle_yy() == bvec2(false, false)

bvec2 test_bvec2_assign_swizzle_from_expression() {
    bvec2 a = bvec2(false, false);
    a.xy = not(bvec2(false, true));
    return a;
    // Should be bvec2(true, false)
}

// run: test_bvec2_assign_swizzle_from_expression() == bvec2(true, false)

bvec2 test_bvec2_assign_swizzle_mixed_names() {
    // Test different name sets (xy, rg, st)
    bvec2 a = bvec2(false, false);
    bvec2 source = bvec2(true, false);
    a.rg = source; // Same as xy
    return a;
    // Should be bvec2(true, false)
}

// run: test_bvec2_assign_swizzle_mixed_names() == bvec2(true, false)
