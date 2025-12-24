// test run
// target riscv32.fixed32

// ============================================================================
// Continue in while loops (jumps to condition)
// ============================================================================

int test_continue_while_loop_skip() {
    int sum = 0;
    int i = 0;
    while (i < 5) {
        if (i == 2) {
            i = i + 1;
            continue;
        }
        sum = sum + i;
        i = i + 1;
    }
    return sum;
    // Should be 0 + 1 + 3 + 4 = 8 (skips i=2)
}

// run: test_continue_while_loop_skip() == 8

int test_continue_while_loop_multiple() {
    int sum = 0;
    int i = 0;
    while (i < 10) {
        if (i % 2 == 0) {
            i = i + 1;
            continue;
        }
        sum = sum + i;
        i = i + 1;
    }
    return sum;
    // Should be 1 + 3 + 5 + 7 + 9 = 25 (skips even numbers)
}

// run: test_continue_while_loop_multiple() == 25

int test_continue_while_loop_early() {
    int sum = 0;
    int i = 0;
    while (i < 5) {
        if (i < 2) {
            i = i + 1;
            continue;
        }
        sum = sum + i;
        i = i + 1;
    }
    return sum;
    // Should be 2 + 3 + 4 = 9 (skips i=0,1)
}

// run: test_continue_while_loop_early() == 9

