// test run
// target riscv32.fixed32

// ============================================================================
// Conversion: float(bool) - converts bool to float (false -> 0.0, true -> 1.0)
// ============================================================================

float test_float_from_bool_false() {
    bool b = false;
    return float(b);
    // Should be 0.0 (false converts to 0.0)
}

// run: test_float_from_bool_false() ~= 0.0

float test_float_from_bool_true() {
    bool b = true;
    return float(b);
    // Should be 1.0 (true converts to 1.0)
}

// run: test_float_from_bool_true() ~= 1.0

float test_float_from_bool_literal_false() {
    return float(false);
    // Should be 0.0
}

// run: test_float_from_bool_literal_false() ~= 0.0

float test_float_from_bool_literal_true() {
    return float(true);
    // Should be 1.0
}

// run: test_float_from_bool_literal_true() ~= 1.0

float test_float_from_bool_expression() {
    bool a = true;
    bool b = true;
    return float(a && b);
    // Should be 1.0 (true && true = true -> 1.0)
}

// run: test_float_from_bool_expression() ~= 1.0

float test_float_from_bool_expression_false() {
    bool a = true;
    bool b = false;
    return float(a && b);
    // Should be 0.0 (true && false = false -> 0.0)
}

// run: test_float_from_bool_expression_false() ~= 0.0

float test_float_from_bool_not() {
    bool a = false;
    return float(!a);
    // Should be 1.0 (!false = true -> 1.0)
}

// run: test_float_from_bool_not() ~= 1.0

float test_float_from_bool_comparison() {
    float x = 3.5;
    float y = 2.1;
    return float(x > y);
    // Should be 1.0 (3.5 > 2.1 = true -> 1.0)
}

// run: test_float_from_bool_comparison() ~= 1.0

