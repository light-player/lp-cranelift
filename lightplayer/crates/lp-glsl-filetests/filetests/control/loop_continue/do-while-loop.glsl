// test run
// target riscv32.fixed32

// ============================================================================
// Continue in do-while loops (jumps to condition)
// ============================================================================

int test_continue_do_while_loop_skip() {
    int sum = 0;
    int i = 0;
    do {
        if (i == 2) {
            i = i + 1;
            continue;
        }
        sum = sum + i;
        i = i + 1;
    } while (i < 5);
    return sum;
    // Should be 0 + 1 + 3 + 4 = 8 (skips i=2)
}

// run: test_continue_do_while_loop_skip() == 8

int test_continue_do_while_loop_multiple() {
    int sum = 0;
    int i = 0;
    do {
        if (i % 2 == 0) {
            i = i + 1;
            continue;
        }
        sum = sum + i;
        i = i + 1;
    } while (i < 10);
    return sum;
    // Should be 1 + 3 + 5 + 7 + 9 = 25 (skips even numbers)
}

// run: test_continue_do_while_loop_multiple() == 25

int test_continue_do_while_loop_after_first() {
    int sum = 0;
    int i = 0;
    do {
        sum = sum + i;
        i = i + 1;
        if (i >= 2) {
            continue;
        }
    } while (i < 5);
    return sum;
    // Should be 0 + 1 = 1 (continues after i=1)
}

// run: test_continue_do_while_loop_after_first() == 1

