// test run
// target riscv32.fixed32

// ============================================================================
// Mix: mix(bvec2, bvec2, bvec2) -> bvec2 (component-wise selection)
// ============================================================================

bvec2 test_bvec2_mix_all_false_selector() {
    bvec2 a = bvec2(true, false);
    bvec2 b = bvec2(false, true);
    bvec2 selector = bvec2(false, false);
    // Function mix() returns bvec2 (component-wise selection)
    // For each component: if selector is false, take from first arg; if true, take from second arg
    return mix(a, b, selector);
    // Should be bvec2(true, false) (all false selector takes all from first arg)
}

// run: test_bvec2_mix_all_false_selector() == bvec2(true, false)

bvec2 test_bvec2_mix_all_true_selector() {
    bvec2 a = bvec2(true, false);
    bvec2 b = bvec2(false, true);
    bvec2 selector = bvec2(true, true);
    return mix(a, b, selector);
    // Should be bvec2(false, true) (all true selector takes all from second arg)
}

// run: test_bvec2_mix_all_true_selector() == bvec2(false, true)

bvec2 test_bvec2_mix_mixed_selector() {
    bvec2 a = bvec2(true, false);
    bvec2 b = bvec2(false, true);
    bvec2 selector = bvec2(false, true);
    return mix(a, b, selector);
    // Should be bvec2(true, true) (first component from a, second from b)
}

// run: test_bvec2_mix_mixed_selector() == bvec2(false, false)

bvec2 test_bvec2_mix_other_mixed_selector() {
    bvec2 a = bvec2(false, true);
    bvec2 b = bvec2(true, false);
    bvec2 selector = bvec2(true, false);
    return mix(a, b, selector);
    // Should be bvec2(true, true) (first component from b, second from a)
}

// run: test_bvec2_mix_other_mixed_selector() == bvec2(true, true)

bvec2 test_bvec2_mix_same_vectors() {
    bvec2 a = bvec2(true, true);
    bvec2 selector = bvec2(false, true);
    return mix(a, a, selector);
    // Should be bvec2(true, true) (same vector regardless of selector)
}

// run: test_bvec2_mix_same_vectors() == bvec2(true, true)

bvec2 test_bvec2_mix_in_expression() {
    bvec2 a = bvec2(true, false);
    bvec2 b = bvec2(false, true);
    bvec2 selector = bvec2(true, false);
    bvec2 result = mix(a, b, selector);
    return not(result);
    // Should be bvec2(false, false) (not((false, false)) = (true, true), wait that's wrong)
    // mix((true,false), (false,true), (true,false)) = (false, false)
    // not((false, false)) = (true, true)
}

// run: test_bvec2_mix_in_expression() == bvec2(true, true)
