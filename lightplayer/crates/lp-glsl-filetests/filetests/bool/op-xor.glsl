// test run
// target riscv32.fixed32

// ============================================================================
// Logical XOR: ^^ operator - logical exclusive or on scalar boolean expressions
// ============================================================================

bool test_bool_xor_true_true() {
    return true ^^ true;
    // Should be false
}

// run: test_bool_xor_true_true() == false

bool test_bool_xor_true_false() {
    return true ^^ false;
    // Should be true
}

// run: test_bool_xor_true_false() == true

bool test_bool_xor_false_true() {
    return false ^^ true;
    // Should be true
}

// run: test_bool_xor_false_true() == true

bool test_bool_xor_false_false() {
    return false ^^ false;
    // Should be false
}

// run: test_bool_xor_false_false() == false

bool test_bool_xor_variables() {
    bool a = true;
    bool b = true;
    return a ^^ b;
    // Should be false
}

// run: test_bool_xor_variables() == false

bool test_bool_xor_different() {
    bool a = true;
    bool b = false;
    return a ^^ b;
    // Should be true
}

// run: test_bool_xor_different() == true

bool test_bool_xor_complex() {
    bool a = true;
    bool b = false;
    bool c = true;
    return (a ^^ b) ^^ c;
    // Should be false (true ^^ false = true, then true ^^ true = false)
}

// run: test_bool_xor_complex() == false

bool test_bool_xor_identity() {
    bool a = true;
    bool b = false;
    // XOR with false is identity
    return a ^^ false;
    // Should be true
}

// run: test_bool_xor_identity() == true

