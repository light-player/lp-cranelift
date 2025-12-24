// test run
// target riscv32.fixed32

// ============================================================================
// Mix: mix(bvec4, bvec4, bvec4) -> bvec4 (component-wise selection)
// ============================================================================

bvec4 test_bvec4_mix_all_false_selector() {
    bvec4 a = bvec4(true, false, true, false);
    bvec4 b = bvec4(false, true, false, true);
    bvec4 selector = bvec4(false, false, false, false);
    // Function mix() returns bvec4 (component-wise selection)
    // For each component: if selector is false, take from first arg; if true, take from second arg
    return mix(a, b, selector);
    // Should be bvec4(true, false, true, false) (all false selector takes all from first arg)
}

// run: test_bvec4_mix_all_false_selector() == bvec4(true, false, true, false)

bvec4 test_bvec4_mix_all_true_selector() {
    bvec4 a = bvec4(true, false, true, false);
    bvec4 b = bvec4(false, true, false, true);
    bvec4 selector = bvec4(true, true, true, true);
    return mix(a, b, selector);
    // Should be bvec4(false, true, false, true) (all true selector takes all from second arg)
}

// run: test_bvec4_mix_all_true_selector() == bvec4(false, true, false, true)

bvec4 test_bvec4_mix_mixed_selector() {
    bvec4 a = bvec4(true, false, true, false);
    bvec4 b = bvec4(false, true, false, true);
    bvec4 selector = bvec4(false, true, false, true);
    return mix(a, b, selector);
    // Should be bvec4(true, true, true, true) (first and third from a, second and fourth from b)
}

// run: test_bvec4_mix_mixed_selector() == bvec4(true, true, true, true)

bvec4 test_bvec4_mix_other_mixed_selector() {
    bvec4 a = bvec4(false, true, false, true);
    bvec4 b = bvec4(true, false, true, false);
    bvec4 selector = bvec4(true, false, true, false);
    return mix(a, b, selector);
    // Should be bvec4(true, true, true, true) (first and third from b, second and fourth from a)
}

// run: test_bvec4_mix_other_mixed_selector() == bvec4(true, false, true, false)

bvec4 test_bvec4_mix_same_vectors() {
    bvec4 a = bvec4(true, true, true, true);
    bvec4 selector = bvec4(false, true, false, true);
    return mix(a, a, selector);
    // Should be bvec4(true, true, true, true) (same vector regardless of selector)
}

// run: test_bvec4_mix_same_vectors() == bvec4(true, true, true, true)

bvec4 test_bvec4_mix_in_expression() {
    bvec4 a = bvec4(true, false, true, false);
    bvec4 b = bvec4(false, true, false, true);
    bvec4 selector = bvec4(true, false, true, false);
    bvec4 result = mix(a, b, selector);
    return not(result);
    // Should be bvec4(true, true, false, true) (not((false, false, false, false)) = (true, true, true, true), wait that's wrong)
    // mix((true,false,true,false), (false,true,false,true), (true,false,true,false)) = (false, false, false, false)
    // not((false, false, false, false)) = (true, true, true, true)
}

// run: test_bvec4_mix_in_expression() == bvec4(true, true, true, true)
