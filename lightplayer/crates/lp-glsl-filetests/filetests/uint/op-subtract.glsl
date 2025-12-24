// test run
// target riscv32.fixed32

// ============================================================================
// Subtract: uint - uint -> uint
// ============================================================================

uint test_uint_subtract_positive_positive() {
    // Subtraction with positive unsigned integers
    return 10u - 3u;
    // Should be 7u
}

// run: test_uint_subtract_positive_positive() == 7u

uint test_uint_subtract_zero() {
    return 42u - 0u;
    // Should be 42u
}

// run: test_uint_subtract_zero() == 42u

uint test_uint_subtract_variables() {
    uint a = 50u;
    uint b = 15u;
    return a - b;
    // Should be 35u
}

// run: test_uint_subtract_variables() == 35u

uint test_uint_subtract_expressions() {
    return (20u - 5u) - (8u - 3u);
    // Should be 10u
}

// run: test_uint_subtract_expressions() == 10u

uint test_uint_subtract_in_assignment() {
    uint result = 20u;
    result = result - 8u;
    return result;
    // Should be 12u
}

// run: test_uint_subtract_in_assignment() == 12u

uint test_uint_subtract_large_numbers() {
    return 500000u - 200000u;
    // Should be 300000u
}

// run: test_uint_subtract_large_numbers() == 300000u

uint test_uint_subtract_small_numbers() {
    return 5u - 2u;
    // Should be 3u
}

// run: test_uint_subtract_small_numbers() == 3u
