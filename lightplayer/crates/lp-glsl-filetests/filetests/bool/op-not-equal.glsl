// test run
// target riscv32.fixed32

// ============================================================================
// Not Equal: != operator - operates on scalar boolean expressions
// ============================================================================

bool test_bool_not_equal_true_true() {
    return true != true;
    // Should be false
}

// run: test_bool_not_equal_true_true() == false

bool test_bool_not_equal_true_false() {
    return true != false;
    // Should be true
}

// run: test_bool_not_equal_true_false() == true

bool test_bool_not_equal_false_true() {
    return false != true;
    // Should be true
}

// run: test_bool_not_equal_false_true() == true

bool test_bool_not_equal_false_false() {
    return false != false;
    // Should be false
}

// run: test_bool_not_equal_false_false() == false

bool test_bool_not_equal_variables_same() {
    bool a = true;
    bool b = true;
    return a != b;
    // Should be false
}

// run: test_bool_not_equal_variables_same() == false

bool test_bool_not_equal_variables_different() {
    bool a = true;
    bool b = false;
    return a != b;
    // Should be true
}

// run: test_bool_not_equal_variables_different() == true

bool test_bool_not_equal_self() {
    bool a = false;
    return a != a;
    // Should be false
}

// run: test_bool_not_equal_self() == false

bool test_bool_not_equal_after_assignment() {
    bool a = true;
    bool b = false;
    b = a;
    return a != b;
    // Should be false
}

// run: test_bool_not_equal_after_assignment() == false

bool test_bool_not_equal_in_expression() {
    bool a = true;
    bool b = false;
    bool c = true;
    return (a != b) && (b != c);
    // Should be true ((true != false) && (false != true) = true && true = true)
}

// run: test_bool_not_equal_in_expression() == true

