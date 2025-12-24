// test run
// target riscv32.fixed32

// ============================================================================
// Multiply: float * float -> float
// ============================================================================

float test_float_multiply_positive_positive() {
    // Multiplication with positive numbers
    return 3.0 * 4.0;
    // Should be 12.0
}

// run: test_float_multiply_positive_positive() ~= 12.0

float test_float_multiply_positive_negative() {
    return 5.0 * (-2.0);
    // Should be -10.0
}

// run: test_float_multiply_positive_negative() ~= -10.0

float test_float_multiply_negative_negative() {
    return (-2.5) * (-2.0);
    // Should be 5.0
}

// run: test_float_multiply_negative_negative() ~= 5.0

float test_float_multiply_by_zero() {
    return 3.14 * 0.0;
    // Should be 0.0
}

// run: test_float_multiply_by_zero() ~= 0.0

float test_float_multiply_by_one() {
    return 7.5 * 1.0;
    // Should be 7.5
}

// run: test_float_multiply_by_one() ~= 7.5

float test_float_multiply_variables() {
    float a = 6.0;
    float b = 7.0;
    return a * b;
    // Should be 42.0
}

// run: test_float_multiply_variables() ~= 42.0

float test_float_multiply_expressions() {
    return (2.0 * 3.0) * (4.0 * 5.0);
    // Should be 120.0
}

// run: test_float_multiply_expressions() ~= 120.0

float test_float_multiply_in_assignment() {
    float result = 2.0;
    result = result * 3.5;
    return result;
    // Should be 7.0
}

// run: test_float_multiply_in_assignment() ~= 7.0

float test_float_multiply_fractions() {
    return 0.5 * 0.25;
    // Should be 0.125
}

// run: test_float_multiply_fractions() ~= 0.125

float test_float_multiply_large_numbers() {
    return 1000.0 * 2000.0;
    // Should be 2000000.0
}

// run: test_float_multiply_large_numbers() ~= 2000000.0
