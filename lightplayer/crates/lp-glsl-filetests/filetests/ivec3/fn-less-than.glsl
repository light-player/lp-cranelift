// test run
// target riscv32.fixed32

// ============================================================================
// Less Than: lessThan(ivec3, ivec3) -> bvec3 (component-wise)
// ============================================================================

bvec3 test_ivec3_less_than_mixed() {
    // Function lessThan() returns bvec3 (component-wise comparison)
    ivec3 a = ivec3(5, 8, 3);
    ivec3 b = ivec3(7, 6, 4);
    return lessThan(a, b);
    // Should be bvec3(true, false, true) (5<7, 8<6=false, 3<4)
}

// run: test_ivec3_less_than_mixed() == bvec3(true, false, true)

bvec3 test_ivec3_less_than_all_true() {
    ivec3 a = ivec3(1, 2, 3);
    ivec3 b = ivec3(5, 6, 7);
    return lessThan(a, b);
    // Should be bvec3(true, true, true)
}

// run: test_ivec3_less_than_all_true() == bvec3(true, true, true)

bvec3 test_ivec3_less_than_all_false() {
    ivec3 a = ivec3(5, 6, 7);
    ivec3 b = ivec3(1, 2, 3);
    return lessThan(a, b);
    // Should be bvec3(false, false, false)
}

// run: test_ivec3_less_than_all_false() == bvec3(false, false, false)

bvec3 test_ivec3_less_than_equal() {
    ivec3 a = ivec3(5, 5, 5);
    ivec3 b = ivec3(5, 6, 4);
    return lessThan(a, b);
    // Should be bvec3(false, true, false) (5<5=false, 5<6, 5<4=false)
}

// run: test_ivec3_less_than_equal() == bvec3(false, true, false)

bvec3 test_ivec3_less_than_negative() {
    ivec3 a = ivec3(-5, -3, -7);
    ivec3 b = ivec3(-3, -5, -2);
    return lessThan(a, b);
    // Should be bvec3(true, false, true) (-5<-3, -3<-5=false, -7<-2)
}

// run: test_ivec3_less_than_negative() == bvec3(true, false, true)

bvec3 test_ivec3_less_than_variables() {
    ivec3 a = ivec3(10, 15, 8);
    ivec3 b = ivec3(12, 10, 12);
    return lessThan(a, b);
    // Should be bvec3(true, false, true)
}

// run: test_ivec3_less_than_variables() == bvec3(true, false, true)

bvec3 test_ivec3_less_than_expressions() {
    return lessThan(ivec3(3, 7, 2), ivec3(5, 5, 4));
    // Should be bvec3(true, false, true)
}

// run: test_ivec3_less_than_expressions() == bvec3(true, false, true)

bvec3 test_ivec3_less_than_in_expression() {
    ivec3 a = ivec3(1, 5, 3);
    ivec3 b = ivec3(2, 3, 4);
    ivec3 c = ivec3(3, 7, 1);
    return lessThan(a, b) == lessThan(b, c);
    // Should be bvec3(true, false, false) (lessThan(a,b)=(true,false,true), lessThan(b,c)=(true,true,false))
    // (true,false,true) == (true,true,false) = (true,false,false)
}

// run: test_ivec3_less_than_in_expression() == bvec3(true, false, false)
