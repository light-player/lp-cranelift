// test run
// target riscv32.fixed32

// ============================================================================
// Add: float + float -> float
// ============================================================================

float test_float_add_positive_positive() {
    // Addition with positive numbers
    return 1.5 + 2.5;
    // Should be 4.0
}

// run: test_float_add_positive_positive() ~= 4.0

float test_float_add_positive_negative() {
    return 5.0 + (-3.0);
    // Should be 2.0
}

// run: test_float_add_positive_negative() ~= 2.0

float test_float_add_negative_negative() {
    return (-2.5) + (-1.5);
    // Should be -4.0
}

// run: test_float_add_negative_negative() ~= -4.0

float test_float_add_zero() {
    return 3.14 + 0.0;
    // Should be 3.14
}

// run: test_float_add_zero() ~= 3.14

float test_float_add_variables() {
    float a = 10.5;
    float b = 20.3;
    return a + b;
    // Should be 30.8
}

// run: test_float_add_variables() ~= 30.8

float test_float_add_expressions() {
    return (2.0 + 3.0) + (4.0 + 5.0);
    // Should be 14.0
}

// run: test_float_add_expressions() ~= 14.0

float test_float_add_in_assignment() {
    float result = 1.0;
    result = result + 2.5;
    return result;
    // Should be 3.5
}

// run: test_float_add_in_assignment() ~= 3.5

float test_float_add_small_numbers() {
    return 0.1 + 0.2;
    // Should be 0.3 (but floating-point precision may affect this)
}

// run: test_float_add_small_numbers() ~= 0.3

float test_float_add_large_numbers() {
    return 1000000.0 + 2000000.0;
    // Should be 3000000.0
}

// run: test_float_add_large_numbers() ~= 3000000.0
