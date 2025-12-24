// test run
// target riscv32.fixed32

// ============================================================================
// Less Equal: int <= int -> bool
// ============================================================================

bool test_int_less_equal_simple() {
    // Simple less than or equal comparison
    return 3 <= 5;
    // Should be true
}

// run: test_int_less_equal_simple() == true

bool test_int_less_equal_equal() {
    return 5 <= 5;
    // Should be true
}

// run: test_int_less_equal_equal() == true

bool test_int_less_equal_greater() {
    return 5 <= 3;
    // Should be false
}

// run: test_int_less_equal_greater() == false

bool test_int_less_equal_negative() {
    return (-5) <= (-3);
    // Should be true
}

// run: test_int_less_equal_negative() == true

bool test_int_less_equal_negative_equal() {
    return (-3) <= (-3);
    // Should be true
}

// run: test_int_less_equal_negative_equal() == true

bool test_int_less_equal_mixed_sign() {
    return (-2) <= 3;
    // Should be true
}

// run: test_int_less_equal_mixed_sign() == true

bool test_int_less_equal_variables() {
    int a = 10;
    int b = 15;
    return a <= b;
    // Should be true
}

// run: test_int_less_equal_variables() == true

bool test_int_less_equal_variables_equal() {
    int a = 10;
    int b = 10;
    return a <= b;
    // Should be true
}

// run: test_int_less_equal_variables_equal() == true

bool test_int_less_equal_expressions() {
    return (2 + 3) <= (6 - 1);
    // Should be true (5 <= 5)
}

// run: test_int_less_equal_expressions() == true

bool test_int_less_equal_zero() {
    return 0 <= 0;
    // Should be true
}

// run: test_int_less_equal_zero() == true
