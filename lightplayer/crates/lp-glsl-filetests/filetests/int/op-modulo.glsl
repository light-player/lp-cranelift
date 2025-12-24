// test run
// target riscv32.fixed32

// ============================================================================
// Modulo: int % int -> int (sign follows dividend)
// ============================================================================

int test_int_modulo_positive_positive() {
    // Modulo operation (sign follows dividend)
    return 10 % 3;
    // Should be 1
}

// run: test_int_modulo_positive_positive() == 1

int test_int_modulo_positive_negative() {
    return 10 % (-3);
    // Should be 1
}

// run: test_int_modulo_positive_negative() == 1

int test_int_modulo_negative_negative() {
    return (-10) % (-3);
    // Should be -1
}

// run: test_int_modulo_negative_negative() == -1

int test_int_modulo_exact_division() {
    return 15 % 5;
    // Should be 0
}

// run: test_int_modulo_exact_division() == 0

int test_int_modulo_variables() {
    int a = 17;
    int b = 5;
    return a % b;
    // Should be 2
}

// run: test_int_modulo_variables() == 2

int test_int_modulo_expressions() {
    return (20 % 7) % 3;
    // Should be 2 (20 % 7 = 6, 6 % 3 = 0, wait that's wrong)
    // Let me fix: 20 % 7 = 6, then 6 % 3 = 0
}

// run: test_int_modulo_expressions() == 0

int test_int_modulo_in_assignment() {
    int result = 25;
    result = result % 7;
    return result;
    // Should be 4
}

// run: test_int_modulo_in_assignment() == 4

int test_int_modulo_negative_dividend() {
    return (-17) % 5;
    // Should be -2
}

// run: test_int_modulo_negative_dividend() == -2

int test_int_modulo_negative_divisor() {
    return 17 % (-5);
    // Should be 2
}

// run: test_int_modulo_negative_divisor() == 2
