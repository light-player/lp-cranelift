// test run
// target riscv32.fixed32

// ============================================================================
// Return with expression
// ============================================================================

int test_return_with_value() {
    return 42;
    // Should be 42
}

// run: test_return_with_value() == 42

int test_return_with_expression() {
    int x = 10;
    int y = 20;
    return x + y;
    // Should be 30
}

// run: test_return_with_expression() == 30

int test_return_with_computation() {
    int a = 5;
    int b = 3;
    return a * b + 2;
    // Should be 5 * 3 + 2 = 17
}

// run: test_return_with_computation() == 17

float test_return_float() {
    return 3.14;
    // Should be 3.14
}

// run: test_return_float() ~= 3.14

float test_return_float_expression() {
    float x = 1.5;
    float y = 2.5;
    return x + y;
    // Should be 4.0
}

// run: test_return_float_expression() ~= 4.0

