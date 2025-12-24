// test run
// target riscv32.fixed32

// ============================================================================
// Control flow: if (bool) - conditions must evaluate to bool
// ============================================================================

int test_if_bool_true() {
    bool b = true;
    int x = 0;
    if (b) {
        x = 10;
    }
    return x;
    // Should be 10
}

// run: test_if_bool_true() == 10

int test_if_bool_false() {
    bool b = false;
    int x = 0;
    if (b) {
        x = 10;
    }
    return x;
    // Should be 0
}

// run: test_if_bool_false() == 0

int test_if_bool_literal_true() {
    int x = 0;
    if (true) {
        x = 5;
    }
    return x;
    // Should be 5
}

// run: test_if_bool_literal_true() == 5

int test_if_bool_literal_false() {
    int x = 0;
    if (false) {
        x = 5;
    }
    return x;
    // Should be 0
}

// run: test_if_bool_literal_false() == 0

int test_if_bool_expression() {
    bool a = true;
    bool b = false;
    int x = 0;
    if (a && b) {
        x = 10;
    }
    return x;
    // Should be 0 (true && false = false)
}

// run: test_if_bool_expression() == 0

int test_if_bool_expression_true() {
    bool a = true;
    bool b = true;
    int x = 0;
    if (a && b) {
        x = 10;
    }
    return x;
    // Should be 10 (true && true = true)
}

// run: test_if_bool_expression_true() == 10

int test_if_bool_not() {
    bool a = false;
    int x = 0;
    if (!a) {
        x = 7;
    }
    return x;
    // Should be 7 (!false = true)
}

// run: test_if_bool_not() == 7

int test_if_bool_comparison() {
    int a = 5;
    int b = 3;
    int x = 0;
    if (a > b) {  // Comparison returns bool
        x = 20;
    }
    return x;
    // Should be 20 (5 > 3 = true)
}

// run: test_if_bool_comparison() == 20

