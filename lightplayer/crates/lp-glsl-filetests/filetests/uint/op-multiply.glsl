// test run
// target riscv32.fixed32

// ============================================================================
// Multiply: uint * uint -> uint
// ============================================================================

uint test_uint_multiply_positive_positive() {
    // Multiplication with positive unsigned integers
    return 6u * 7u;
    // Should be 42u
}

// run: test_uint_multiply_positive_positive() == 42u

uint test_uint_multiply_by_zero() {
    return 123u * 0u;
    // Should be 0u
}

// run: test_uint_multiply_by_zero() == 0u

uint test_uint_multiply_by_one() {
    return 42u * 1u;
    // Should be 42u
}

// run: test_uint_multiply_by_one() == 42u

uint test_uint_multiply_variables() {
    uint a = 8u;
    uint b = 9u;
    return a * b;
    // Should be 72u
}

// run: test_uint_multiply_variables() == 72u

uint test_uint_multiply_expressions() {
    return (3u * 4u) * (2u * 5u);
    // Should be 120u
}

// run: test_uint_multiply_expressions() == 120u

uint test_uint_multiply_in_assignment() {
    uint result = 6u;
    result = result * 7u;
    return result;
    // Should be 42u
}

// run: test_uint_multiply_in_assignment() == 42u

uint test_uint_multiply_large_numbers() {
    return 1000u * 2000u;
    // Should be 2000000u
}

// run: test_uint_multiply_large_numbers() == 2000000u

uint test_uint_multiply_small_numbers() {
    return 2u * 3u;
    // Should be 6u
}

// run: test_uint_multiply_small_numbers() == 6u
