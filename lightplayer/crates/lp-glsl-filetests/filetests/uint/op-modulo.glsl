// test run
// target riscv32.fixed32

// ============================================================================
// Modulo: uint % uint -> uint
// ============================================================================

uint test_uint_modulo_positive_positive() {
    // Modulo operation with positive unsigned integers
    return 10u % 3u;
    // Should be 1u
}

// run: test_uint_modulo_positive_positive() == 1u

uint test_uint_modulo_exact_division() {
    return 15u % 5u;
    // Should be 0u
}

// run: test_uint_modulo_exact_division() == 0u

uint test_uint_modulo_variables() {
    uint a = 17u;
    uint b = 5u;
    return a % b;
    // Should be 2u
}

// run: test_uint_modulo_variables() == 2u

uint test_uint_modulo_expressions() {
    return (20u % 7u) % 3u;
    // Should be 2u (20u % 7u = 6u, 6u % 3u = 0u, wait that's wrong)
    // Let me fix: 20u % 7u = 6u, then 6u % 3u = 0u
}

// run: test_uint_modulo_expressions() == 0u

uint test_uint_modulo_in_assignment() {
    uint result = 25u;
    result = result % 7u;
    return result;
    // Should be 4u
}

// run: test_uint_modulo_in_assignment() == 4u

uint test_uint_modulo_large_numbers() {
    return 1000000u % 1000u;
    // Should be 0u
}

// run: test_uint_modulo_large_numbers() == 0u

uint test_uint_modulo_remainder() {
    return 12345u % 1000u;
    // Should be 345u
}

// run: test_uint_modulo_remainder() == 345u
