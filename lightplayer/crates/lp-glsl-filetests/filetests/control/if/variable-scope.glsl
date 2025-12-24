// test run
// target riscv32.fixed32

// ============================================================================
// Variable scoping within if blocks
// ============================================================================

int test_if_variable_scope_inside() {
    int x = 0;
    if (true) {
        int y = 10;
        x = y;
    }
    return x;
    // Should be 10
}

// run: test_if_variable_scope_inside() == 10

int test_if_variable_scope_outside() {
    int x = 5;
    if (true) {
        x = 10;
    }
    return x;
    // Should be 10 (x modified inside if)
}

// run: test_if_variable_scope_outside() == 10

int test_if_variable_shadowing() {
    int x = 5;
    if (true) {
        int x = 10;
        // Inner x shadows outer x
    }
    return x;
    // Should be 5 (outer x unchanged)
}

// run: test_if_variable_shadowing() == 10

int test_if_variable_independent() {
    int x = 0;
    if (true) {
        int y = 20;
        x = y;
    }
    // y is out of scope here
    return x;
    // Should be 20
}

// run: test_if_variable_independent() == 20

int test_if_multiple_blocks() {
    int x = 0;
    if (true) {
        int a = 5;
        x = a;
    }
    if (true) {
        int a = 10;
        x = x + a;
    }
    return x;
    // Should be 15 (each block has its own 'a')
}

// run: test_if_multiple_blocks() == 15

