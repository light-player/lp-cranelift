// test run
// target riscv32.fixed32

// ============================================================================
// Constructor: bool(uint) - converts uint to bool (0u -> false, non-zero -> true)
// ============================================================================

bool test_bool_from_uint_zero() {
    uint u = 0u;
    return bool(u);
    // Should be false (0u converts to false)
}

// run: test_bool_from_uint_zero() == false

bool test_bool_from_uint_positive() {
    uint u = 42u;
    return bool(u);
    // Should be true (non-zero converts to true)
}

// run: test_bool_from_uint_positive() == true

bool test_bool_from_uint_one() {
    uint u = 1u;
    return bool(u);
    // Should be true
}

// run: test_bool_from_uint_one() == true

bool test_bool_from_uint_literal_zero() {
    return bool(0u);
    // Should be false
}

// run: test_bool_from_uint_literal_zero() == false

bool test_bool_from_uint_literal_nonzero() {
    return bool(7u);
    // Should be true
}

// run: test_bool_from_uint_literal_nonzero() == true

bool test_bool_from_uint_expression() {
    uint a = 10u;
    uint b = 3u;
    return bool(a - b);
    // Should be true (10u - 3u = 7u, non-zero)
}

// run: test_bool_from_uint_expression() == true

bool test_bool_from_uint_expression_zero() {
    uint a = 8u;
    uint b = 8u;
    return bool(a - b);
    // Should be false (8u - 8u = 0u)
}

// run: test_bool_from_uint_expression_zero() == false

bool test_bool_from_uint_large() {
    uint u = 4294967295u;  // Max uint value
    return bool(u);
    // Should be true (non-zero)
}

// run: test_bool_from_uint_large() == true

