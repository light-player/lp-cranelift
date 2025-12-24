// test run
// target riscv32.fixed32

// ============================================================================
// Edge cases: Boolean literals - true and false constants
// ============================================================================

bool test_bool_literal_true() {
    return true;
    // Should be true
}

// run: test_bool_literal_true() == true

bool test_bool_literal_false() {
    return false;
    // Should be false
}

// run: test_bool_literal_false() == false

bool test_bool_literal_in_expression() {
    return true && false;
    // Should be false
}

// run: test_bool_literal_in_expression() == false

bool test_bool_literal_in_comparison() {
    return true == true;
    // Should be true
}

// run: test_bool_literal_in_comparison() == true

bool test_bool_literal_assignment() {
    bool a = true;
    bool b = false;
    return a && !b;
    // Should be true (true && !false = true && true = true)
}

// run: test_bool_literal_assignment() == true

bool test_bool_literal_constructor() {
    return bool(true);
    // Should be true
}

// run: test_bool_literal_constructor() == true

bool test_bool_literal_conversion() {
    return bool(1) == true;
    // Should be true (bool(1) = true)
}

// run: test_bool_literal_conversion() == true

bool test_bool_literal_ternary() {
    return true ? true : false;
    // Should be true
}

// run: test_bool_literal_ternary() == true

bool test_bool_literal_complex() {
    return (true && false) || (false || true);
    // Should be true ((true && false) || (false || true) = false || true = true)
}

// run: test_bool_literal_complex() == true

