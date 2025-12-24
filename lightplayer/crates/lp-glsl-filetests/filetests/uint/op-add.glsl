// test run
// target riscv32.fixed32

// ============================================================================
// Add: uint + uint -> uint
// ============================================================================

uint test_uint_add_positive_positive() {
    // Addition with positive unsigned integers
    return 5u + 3u;
    // Should be 8u
}

// run: test_uint_add_positive_positive() == 8u

uint test_uint_add_zero() {
    return 42u + 0u;
    // Should be 42u
}

// run: test_uint_add_zero() == 42u

uint test_uint_add_variables() {
    uint a = 15u;
    uint b = 27u;
    return a + b;
    // Should be 42u
}

// run: test_uint_add_variables() == 42u

uint test_uint_add_expressions() {
    return (8u + 4u) + (6u + 2u);
    // Should be 20u
}

// run: test_uint_add_expressions() == 20u

uint test_uint_add_in_assignment() {
    uint result = 5u;
    result = result + 10u;
    return result;
    // Should be 15u
}

// run: test_uint_add_in_assignment() == 15u

uint test_uint_add_large_numbers() {
    return 100000u + 200000u;
    // Should be 300000u
}

// run: test_uint_add_large_numbers() == 300000u

uint test_uint_add_small_numbers() {
    return 1u + 2u;
    // Should be 3u
}

// run: test_uint_add_small_numbers() == 3u
