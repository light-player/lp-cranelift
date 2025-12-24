// test run
// target riscv32.fixed32

// ============================================================================
// Basic while loops
// ============================================================================

int test_while_loop_basic() {
    int i = 0;
    int sum = 0;
    while (i < 5) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
    // Should be 0 + 1 + 2 + 3 + 4 = 10
}

// run: test_while_loop_basic() == 10

int test_while_loop_count() {
    int count = 0;
    int i = 0;
    while (i < 10) {
        count = count + 1;
        i = i + 1;
    }
    return count;
    // Should be 10
}

// run: test_while_loop_count() == 10

int test_while_loop_zero_iterations() {
    int sum = 0;
    int i = 0;
    while (i < 0) {
        sum = sum + 1;
        i = i + 1;
    }
    return sum;
    // Should be 0 (loop doesn't execute)
}

// run: test_while_loop_zero_iterations() == 0

int test_while_loop_single_iteration() {
    int sum = 0;
    int i = 0;
    while (i < 1) {
        sum = sum + 1;
        i = i + 1;
    }
    return sum;
    // Should be 1
}

// run: test_while_loop_single_iteration() == 1

int test_while_loop_decrement() {
    int sum = 0;
    int i = 5;
    while (i > 0) {
        sum = sum + i;
        i = i - 1;
    }
    return sum;
    // Should be 5 + 4 + 3 + 2 + 1 = 15
}

// run: test_while_loop_decrement() == 15

