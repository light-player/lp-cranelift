// test run
// target riscv32.fixed32

// ============================================================================
// Logical NOT: ! operator - operates on scalar boolean expressions
// ============================================================================

bool test_bool_not_true() {
    return !true;
    // Should be false
}

// run: test_bool_not_true() == false

bool test_bool_not_false() {
    return !false;
    // Should be true
}

// run: test_bool_not_false() == true

bool test_bool_not_variable_true() {
    bool a = true;
    return !a;
    // Should be false
}

// run: test_bool_not_variable_true() == false

bool test_bool_not_variable_false() {
    bool a = false;
    return !a;
    // Should be true
}

// run: test_bool_not_variable_false() == true

bool test_bool_not_double_negation() {
    bool a = true;
    return !!a;
    // Should be true (double negation)
}

// run: test_bool_not_double_negation() == true

bool test_bool_not_triple_negation() {
    bool a = false;
    return !!!a;
    // Should be true (triple negation: !false = true, !true = false, !false = true)
}

// run: test_bool_not_triple_negation() == true

bool test_bool_not_in_expression() {
    bool a = true;
    bool b = false;
    return !a && !b;
    // Should be false (!true && !false = false && true = false)
}

// run: test_bool_not_in_expression() == false

bool test_bool_not_complex() {
    bool a = true;
    bool b = false;
    return !(a && b);
    // Should be true (!(true && false) = !false = true)
}

// run: test_bool_not_complex() == true

