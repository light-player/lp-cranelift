// test run
// target riscv32.fixed32

// ============================================================================
// Conversion: int(bool) - converts bool to int (false -> 0, true -> 1)
// ============================================================================

int test_int_from_bool_false() {
    bool b = false;
    return int(b);
    // Should be 0 (false converts to 0)
}

// run: test_int_from_bool_false() == 0

int test_int_from_bool_true() {
    bool b = true;
    return int(b);
    // Should be 1 (true converts to 1)
}

// run: test_int_from_bool_true() == 1

int test_int_from_bool_literal_false() {
    return int(false);
    // Should be 0
}

// run: test_int_from_bool_literal_false() == 0

int test_int_from_bool_literal_true() {
    return int(true);
    // Should be 1
}

// run: test_int_from_bool_literal_true() == 1

int test_int_from_bool_expression() {
    bool a = true;
    bool b = false;
    return int(a && b);
    // Should be 0 (true && false = false -> 0)
}

// run: test_int_from_bool_expression() == 0

int test_int_from_bool_expression_true() {
    bool a = true;
    bool b = true;
    return int(a && b);
    // Should be 1 (true && true = true -> 1)
}

// run: test_int_from_bool_expression_true() == 1

int test_int_from_bool_not() {
    bool a = false;
    return int(!a);
    // Should be 1 (!false = true -> 1)
}

// run: test_int_from_bool_not() == 1

int test_int_from_bool_comparison() {
    int x = 5;
    int y = 3;
    return int(x > y);
    // Should be 1 (5 > 3 = true -> 1)
}

// run: test_int_from_bool_comparison() == 1

