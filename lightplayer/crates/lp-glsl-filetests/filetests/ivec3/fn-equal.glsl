// test run
// target riscv32.fixed32

// ============================================================================
// Equal: equal(ivec3, ivec3) -> bvec3 (component-wise)
// ============================================================================

bvec3 test_ivec3_equal_function_mixed() {
    ivec3 a = ivec3(5, 3, 7);
    ivec3 b = ivec3(5, 4, 7);
    // Function equal() returns bvec3 (component-wise equality)
    return equal(a, b);
    // Should be bvec3(true, false, true)
}

// run: test_ivec3_equal_function_mixed() == bvec3(true, false, true)

bvec3 test_ivec3_equal_function_all_true() {
    ivec3 a = ivec3(10, 20, 30);
    ivec3 b = ivec3(10, 20, 30);
    return equal(a, b);
    // Should be bvec3(true, true, true)
}

// run: test_ivec3_equal_function_all_true() == bvec3(true, true, true)

bvec3 test_ivec3_equal_function_all_false() {
    ivec3 a = ivec3(5, 3, 7);
    ivec3 b = ivec3(2, 4, 1);
    return equal(a, b);
    // Should be bvec3(false, false, false)
}

// run: test_ivec3_equal_function_all_false() == bvec3(false, false, false)

bvec3 test_ivec3_equal_function_zero() {
    ivec3 a = ivec3(0, 5, 0);
    ivec3 b = ivec3(0, 3, 1);
    return equal(a, b);
    // Should be bvec3(true, false, false)
}

// run: test_ivec3_equal_function_zero() == bvec3(true, false, false)

bvec3 test_ivec3_equal_function_negative() {
    ivec3 a = ivec3(-5, -3, -7);
    ivec3 b = ivec3(-5, -4, -7);
    return equal(a, b);
    // Should be bvec3(true, false, true)
}

// run: test_ivec3_equal_function_negative() == bvec3(true, false, true)

bvec3 test_ivec3_equal_function_variables() {
    ivec3 a = ivec3(8, 12, 6);
    ivec3 b = ivec3(8, 10, 7);
    return equal(a, b);
    // Should be bvec3(true, false, false)
}

// run: test_ivec3_equal_function_variables() == bvec3(true, false, false)

bvec3 test_ivec3_equal_function_expressions() {
    return equal(ivec3(2, 5, 3), ivec3(2, 4, 8));
    // Should be bvec3(true, false, false)
}

// run: test_ivec3_equal_function_expressions() == bvec3(true, false, false)

bvec3 test_ivec3_equal_function_in_expression() {
    ivec3 a = ivec3(1, 3, 5);
    ivec3 b = ivec3(1, 4, 5);
    ivec3 c = ivec3(2, 3, 5);
    return equal(a, b) == equal(b, c);
    // Should be bvec3(true, false, true) (equal(a,b)=(true,false,true), equal(b,c)=(false,false,true))
    // (true,false,true) == (false,false,true) = (false,false,true)
}

// run: test_ivec3_equal_function_in_expression() == bvec3(false, false, true)
