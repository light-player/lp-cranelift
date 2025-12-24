// test run
// target riscv32.fixed32

// ============================================================================
// Constructor: bool(int) - converts int to bool (0 -> false, non-zero -> true)
// ============================================================================

bool test_bool_from_int_zero() {
    int i = 0;
    return bool(i);
    // Should be false (0 converts to false)
}

// run: test_bool_from_int_zero() == false

bool test_bool_from_int_positive() {
    int i = 42;
    return bool(i);
    // Should be true (non-zero converts to true)
}

// run: test_bool_from_int_positive() == true

bool test_bool_from_int_negative() {
    int i = -10;
    return bool(i);
    // Should be true (non-zero converts to true)
}

// run: test_bool_from_int_negative() == true

bool test_bool_from_int_one() {
    int i = 1;
    return bool(i);
    // Should be true
}

// run: test_bool_from_int_one() == true

bool test_bool_from_int_literal_zero() {
    return bool(0);
    // Should be false
}

// run: test_bool_from_int_literal_zero() == false

bool test_bool_from_int_literal_nonzero() {
    return bool(5);
    // Should be true
}

// run: test_bool_from_int_literal_nonzero() == true

bool test_bool_from_int_expression() {
    int a = 3;
    int b = 2;
    return bool(a - b);
    // Should be true (3 - 2 = 1, non-zero)
}

// run: test_bool_from_int_expression() == true

bool test_bool_from_int_expression_zero() {
    int a = 5;
    int b = 5;
    return bool(a - b);
    // Should be false (5 - 5 = 0)
}

// run: test_bool_from_int_expression_zero() == false

