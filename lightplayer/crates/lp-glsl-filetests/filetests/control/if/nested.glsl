// test run
// target riscv32.fixed32

// ============================================================================
// Nested if statements
// ============================================================================

int test_if_nested_both_true() {
    int x = 0;
    if (true) {
        if (true) {
            x = 10;
        }
    }
    return x;
    // Should be 10
}

// run: test_if_nested_both_true() == 10

int test_if_nested_outer_false() {
    int x = 0;
    if (false) {
        if (true) {
            x = 10;
        }
    }
    return x;
    // Should be 0 (outer false prevents execution)
}

// run: test_if_nested_outer_false() == 0

int test_if_nested_inner_false() {
    int x = 0;
    if (true) {
        if (false) {
            x = 10;
        }
    }
    return x;
    // Should be 0 (inner false prevents execution)
}

// run: test_if_nested_inner_false() == 0

int test_if_nested_triple() {
    int x = 0;
    if (true) {
        if (true) {
            if (true) {
                x = 20;
            }
        }
    }
    return x;
    // Should be 20
}

// run: test_if_nested_triple() == 20

int test_if_nested_conditional() {
    int x = 0;
    if (5 > 3) {
        if (10 > 8) {
            x = 15;
        }
    }
    return x;
    // Should be 15
}

// run: test_if_nested_conditional() == 15

