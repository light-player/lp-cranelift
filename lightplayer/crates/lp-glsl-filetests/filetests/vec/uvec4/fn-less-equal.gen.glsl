// This file is GENERATED. Do not edit manually.
// To regenerate, run:
//   lp-filetests-gen vec/uvec4/fn-greater-equal --write
//
// test run
// target riscv32.fixed32

// ============================================================================
// Less Than Equal: lessThanEqual(uvec4, uvec4) -> bvec4 (component-wise)
// ============================================================================

bvec4 test_uvec4_less_equal_mixed() {
// Function lessThanEqual() returns bvec4 (component-wise comparison)
uvec4 a = uvec4(5u, 8u, 7u, 4u);
uvec4 b = uvec4(7u, 6u, 9u, 2u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_mixed() == bvec4(true, false, true, false)

bvec4 test_uvec4_less_equal_all_true() {
uvec4 a = uvec4(1u, 2u, 3u, 4u);
uvec4 b = uvec4(5u, 6u, 7u, 8u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_all_true() == bvec4(true, true, true, true)

bvec4 test_uvec4_less_equal_all_false() {
uvec4 a = uvec4(5u, 6u, 7u, 8u);
uvec4 b = uvec4(1u, 2u, 3u, 4u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_all_false() == bvec4(false, false, false, false)

bvec4 test_uvec4_less_equal_equal() {
uvec4 a = uvec4(5u, 5u, 5u, 5u);
uvec4 b = uvec4(5u, 5u, 5u, 5u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_equal() == bvec4(true, true, true, true)

bvec4 test_uvec4_less_equal_mixed_equal() {
uvec4 a = uvec4(5u, 6u, 7u, 8u);
uvec4 b = uvec4(5u, 5u, 8u, 7u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_mixed_equal() == bvec4(true, false, true, false)

bvec4 test_uvec4_less_equal_zero() {
uvec4 a = uvec4(0u, 1u, 2u, 0u);
uvec4 b = uvec4(1u, 0u, 3u, -1u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_zero() == bvec4(true, false, true, false)

bvec4 test_uvec4_less_equal_variables() {
uvec4 a = uvec4(10u, 15u, 8u, 5u);
uvec4 b = uvec4(12u, 10u, 8u, 6u);
return lessThanEqual(a, b);
}

// run: test_uvec4_less_equal_variables() == bvec4(true, false, true, true)

bvec4 test_uvec4_less_equal_expressions() {
return lessThanEqual(uvec4(3u, 7u, 6u, 4u), uvec4(5u, 5u, 6u, 3u));
}

// run: test_uvec4_less_equal_expressions() == bvec4(true, false, true, false)

bvec4 test_uvec4_less_equal_in_expression() {
uvec4 a = uvec4(1u, 5u, 4u, 7u);
uvec4 b = uvec4(2u, 3u, 6u, 8u);
uvec4 c = uvec4(3u, 7u, 5u, 9u);
// Use equal() for component-wise comparison of bvec4 values
// lessThanEqual(a, b) = (true,false,true,true)
// lessThanEqual(b, c) = (true,true,false,true)
// equal(lessThanEqual(a, b), lessThanEqual(b, c)) = (true,false,false,true)
return equal(lessThanEqual(a, b), lessThanEqual(b, c));
}

// run: test_uvec4_less_equal_in_expression() == bvec4(true, false, false, true)
