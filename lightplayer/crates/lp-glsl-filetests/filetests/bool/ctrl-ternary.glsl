// test run
// target riscv32.fixed32

// ============================================================================
// Ternary operator: bool ? expr1 : expr2 - condition must be bool
// ============================================================================

int test_ternary_bool_true() {
    bool b = true;
    return b ? 10 : 20;
    // Should be 10 (true selects first expression)
}

// run: test_ternary_bool_true() == 10

int test_ternary_bool_false() {
    bool b = false;
    return b ? 10 : 20;
    // Should be 20 (false selects second expression)
}

// run: test_ternary_bool_false() == 20

int test_ternary_bool_literal_true() {
    return true ? 5 : 15;
    // Should be 5
}

// run: test_ternary_bool_literal_true() == 5

int test_ternary_bool_literal_false() {
    return false ? 5 : 15;
    // Should be 15
}

// run: test_ternary_bool_literal_false() == 15

int test_ternary_bool_expression() {
    bool a = true;
    bool b = false;
    return (a && b) ? 100 : 200;
    // Should be 200 (true && false = false)
}

// run: test_ternary_bool_expression() == 200

int test_ternary_bool_not() {
    bool a = false;
    return !a ? 7 : 14;
    // Should be 7 (!false = true)
}

// run: test_ternary_bool_not() == 7

int test_ternary_bool_comparison() {
    int x = 5;
    int y = 3;
    return (x > y) ? 30 : 40;
    // Should be 30 (5 > 3 = true)
}

// run: test_ternary_bool_comparison() == 30

int test_ternary_bool_nested() {
    bool a = true;
    bool b = false;
    return a ? (b ? 1 : 2) : (b ? 3 : 4);
    // Should be 2 (a=true selects first branch, b=false selects 2)
}

// run: test_ternary_bool_nested() == 2

int test_ternary_bool_float_result() {
    bool b = true;
    float result = b ? 1.5 : 2.5;
    return int(result * 2.0);
    // Should be 3 (1.5 * 2.0 = 3.0 -> 3)
}

// run: test_ternary_bool_float_result() == 3

