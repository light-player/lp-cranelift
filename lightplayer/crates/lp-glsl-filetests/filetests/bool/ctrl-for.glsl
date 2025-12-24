// test run
// target riscv32.fixed32

// ============================================================================
// Control flow: for (init; bool; update) - condition must evaluate to bool
// ============================================================================

int test_for_bool_condition() {
    int x = 0;
    bool b = true;
    int count = 0;
    for (int i = 0; b && i < 5; i = i + 1) {
        x = x + 1;
    }
    return x;
    // Should be 5 (loop runs 5 times)
}

// run: test_for_bool_condition() == 5

int test_for_bool_false() {
    int x = 0;
    bool b = false;
    for (int i = 0; b; i = i + 1) {
        x = x + 1;
    }
    return x;
    // Should be 0 (loop never runs)
}

// run: test_for_bool_false() == 0

int test_for_bool_change_condition() {
    int x = 0;
    bool b = true;
    for (int i = 0; b && i < 10; i = i + 1) {
        x = x + 1;
        if (i >= 3) {
            b = false;  // Change condition to exit early
        }
    }
    return x;
    // Should be 4 (loop runs 4 times: i=0,1,2,3, then b becomes false)
}

// run: test_for_bool_change_condition() == 4

int test_for_bool_comparison() {
    int x = 0;
    for (int i = 0; i < 4; i = i + 1) {  // Comparison returns bool
        x = x + 1;
    }
    return x;
    // Should be 4 (loop runs 4 times)
}

// run: test_for_bool_comparison() == 4

int test_for_bool_expression() {
    int x = 0;
    bool a = true;
    bool b = true;
    for (int i = 0; a && b && i < 3; i = i + 1) {
        x = x + 1;
    }
    return x;
    // Should be 3 (loop runs 3 times)
}

// run: test_for_bool_expression() == 3

int test_for_bool_not() {
    int x = 0;
    bool done = false;
    for (int i = 0; !done && i < 5; i = i + 1) {
        x = x + 1;
    }
    return x;
    // Should be 5 (!false = true, loop runs 5 times)
}

// run: test_for_bool_not() == 5

int test_for_bool_early_exit() {
    int x = 0;
    bool continue_loop = true;
    for (int i = 0; continue_loop && i < 10; i = i + 1) {
        x = x + 1;
        if (x >= 6) {
            continue_loop = false;
        }
    }
    return x;
    // Should be 6 (loop runs 6 times then exits)
}

// run: test_for_bool_early_exit() == 6

