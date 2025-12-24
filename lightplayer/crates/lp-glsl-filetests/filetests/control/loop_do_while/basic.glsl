// test run
// target riscv32.fixed32

// ============================================================================
// Basic do-while loops
// ============================================================================

int test_do_while_loop_basic() {
    int i = 0;
    int sum = 0;
    do {
        sum = sum + i;
        i = i + 1;
    } while (i < 5);
    return sum;
    // Should be 0 + 1 + 2 + 3 + 4 = 10
}

// run: test_do_while_loop_basic() == 10

int test_do_while_loop_count() {
    int count = 0;
    int i = 0;
    do {
        count = count + 1;
        i = i + 1;
    } while (i < 10);
    return count;
    // Should be 10
}

// run: test_do_while_loop_count() == 10

int test_do_while_loop_decrement() {
    int sum = 0;
    int i = 5;
    do {
        sum = sum + i;
        i = i - 1;
    } while (i > 0);
    return sum;
    // Should be 5 + 4 + 3 + 2 + 1 = 15
}

// run: test_do_while_loop_decrement() == 15

