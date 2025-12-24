// test run
// target riscv32.fixed32

// ============================================================================
// Mix: mix(bvec3, bvec3, bvec3) -> bvec3 (component-wise selection)
// ============================================================================

bvec3 test_bvec3_mix_all_false_selector() {
    bvec3 a = bvec3(true, false, true);
    bvec3 b = bvec3(false, true, false);
    bvec3 selector = bvec3(false, false, false);
    // Function mix() returns bvec3 (component-wise selection)
    // For each component: if selector is false, take from first arg; if true, take from second arg
    return mix(a, b, selector);
    // Should be bvec3(true, false, true) (all false selector takes all from first arg)
}

// run: test_bvec3_mix_all_false_selector() == bvec3(true, false, true)

bvec3 test_bvec3_mix_all_true_selector() {
    bvec3 a = bvec3(true, false, true);
    bvec3 b = bvec3(false, true, false);
    bvec3 selector = bvec3(true, true, true);
    return mix(a, b, selector);
    // Should be bvec3(false, true, false) (all true selector takes all from second arg)
}

// run: test_bvec3_mix_all_true_selector() == bvec3(false, true, false)

bvec3 test_bvec3_mix_mixed_selector() {
    bvec3 a = bvec3(true, false, true);
    bvec3 b = bvec3(false, true, false);
    bvec3 selector = bvec3(false, true, false);
    return mix(a, b, selector);
    // Should be bvec3(true, true, true) (first from a, second from b, third from a)
}

// run: test_bvec3_mix_mixed_selector() == bvec3(true, true, true)

bvec3 test_bvec3_mix_other_mixed_selector() {
    bvec3 a = bvec3(false, true, false);
    bvec3 b = bvec3(true, false, true);
    bvec3 selector = bvec3(true, false, true);
    return mix(a, b, selector);
    // Should be bvec3(true, true, true) (first from b, second from a, third from b)
}

// run: test_bvec3_mix_other_mixed_selector() == bvec3(true, false, true)

bvec3 test_bvec3_mix_same_vectors() {
    bvec3 a = bvec3(true, true, true);
    bvec3 selector = bvec3(false, true, false);
    return mix(a, a, selector);
    // Should be bvec3(true, true, true) (same vector regardless of selector)
}

// run: test_bvec3_mix_same_vectors() == bvec3(true, true, true)

bvec3 test_bvec3_mix_in_expression() {
    bvec3 a = bvec3(true, false, true);
    bvec3 b = bvec3(false, true, false);
    bvec3 selector = bvec3(true, false, true);
    bvec3 result = mix(a, b, selector);
    return not(result);
    // Should be bvec3(true, true, false) (not((false, false, false)) = (true, true, true), wait that's wrong)
    // mix((true,false,true), (false,true,false), (true,false,true)) = (false, false, false)
    // not((false, false, false)) = (true, true, true)
}

// run: test_bvec3_mix_in_expression() == bvec3(true, true, true)
