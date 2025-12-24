// test run
// target riscv32.fixed32

// ============================================================================
// Equality: == operator - operates on scalar boolean expressions
// ============================================================================

bool test_bool_equal_true_true() {
    return true == true;
    // Should be true
}

// run: test_bool_equal_true_true() == true

bool test_bool_equal_true_false() {
    return true == false;
    // Should be false
}

// run: test_bool_equal_true_false() == false

bool test_bool_equal_false_true() {
    return false == true;
    // Should be false
}

// run: test_bool_equal_false_true() == false

bool test_bool_equal_false_false() {
    return false == false;
    // Should be true
}

// run: test_bool_equal_false_false() == true

bool test_bool_equal_variables_same() {
    bool a = true;
    bool b = true;
    return a == b;
    // Should be true
}

// run: test_bool_equal_variables_same() == true

bool test_bool_equal_variables_different() {
    bool a = true;
    bool b = false;
    return a == b;
    // Should be false
}

// run: test_bool_equal_variables_different() == false

bool test_bool_equal_self() {
    bool a = true;
    return a == a;
    // Should be true
}

// run: test_bool_equal_self() == true

bool test_bool_equal_after_assignment() {
    bool a = true;
    bool b = false;
    b = a;
    return a == b;
    // Should be true
}

// run: test_bool_equal_after_assignment() == true

bool test_bool_equal_in_expression() {
    bool a = true;
    bool b = true;
    bool c = false;
    return (a == b) && (b == c);
    // Should be false ((true == true) && (true == false) = true && false = false)
}

// run: test_bool_equal_in_expression() == false

