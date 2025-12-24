// test run
// target riscv32.fixed32

// ============================================================================
// Less Than: float < float -> bool
// ============================================================================

bool test_float_less_than_simple() {
    // Simple less than comparison
    return 3.0 < 5.0;
    // Should be true
}

// run: test_float_less_than_simple() == true

bool test_float_less_than_equal() {
    return 5.0 < 5.0;
    // Should be false
}

// run: test_float_less_than_equal() == false

bool test_float_less_than_negative() {
    return (-5.0) < (-3.0);
    // Should be true
}

// run: test_float_less_than_negative() == true

bool test_float_less_than_mixed_sign() {
    return (-2.0) < 3.0;
    // Should be true
}

// run: test_float_less_than_mixed_sign() == true

bool test_float_less_than_from_zero() {
    return (-1.0) < 0.0;
    // Should be true
}

// run: test_float_less_than_from_zero() == true

bool test_float_less_than_to_zero() {
    return 0.0 < 1.0;
    // Should be true
}

// run: test_float_less_than_to_zero() == true

bool test_float_less_than_variables() {
    float a = 10.5;
    float b = 15.3;
    return a < b;
    // Should be true
}

// run: test_float_less_than_variables() == true

bool test_float_less_than_expressions() {
    return (2.0 + 3.0) < (6.0 - 1.0);
    // Should be true (5.0 < 5.0 is false)
}

// run: test_float_less_than_expressions() == false

bool test_float_less_than_fractions() {
    return 0.1 < 0.2;
    // Should be true
}

// run: test_float_less_than_fractions() == true

bool test_float_less_than_large_numbers() {
    return 1000000.0 < 2000000.0;
    // Should be true
}

// run: test_float_less_than_large_numbers() == true
