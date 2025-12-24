// test run
// target riscv32.fixed32

// ============================================================================
// Continue in for loops (jumps to loop-expression)
// ============================================================================

int test_continue_for_loop_skip() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        if (i == 2) {
            continue;
        }
        sum = sum + i;
    }
    return sum;
    // Should be 0 + 1 + 3 + 4 = 8 (skips i=2)
}

// run: test_continue_for_loop_skip() == 8

int test_continue_for_loop_multiple() {
    int sum = 0;
    for (int i = 0; i < 10; i++) {
        if (i % 2 == 0) {
            continue;
        }
        sum = sum + i;
    }
    return sum;
    // Should be 1 + 3 + 5 + 7 + 9 = 25 (skips even numbers)
}

// run: test_continue_for_loop_multiple() == 25

int test_continue_for_loop_early() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        if (i < 2) {
            continue;
        }
        sum = sum + i;
    }
    return sum;
    // Should be 2 + 3 + 4 = 9 (skips i=0,1)
}

// run: test_continue_for_loop_early() == 9

int test_continue_for_loop_all() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        continue;
        sum = sum + i;
    }
    return sum;
    // Should be 0 (all iterations skipped)
}

// run: test_continue_for_loop_all() == 0

