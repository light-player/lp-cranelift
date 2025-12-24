// test run
// target riscv32.fixed32

// ============================================================================
// For loops with decrement
// ============================================================================

int test_for_loop_decrement() {
    int sum = 0;
    for (int i = 5; i > 0; i--) {
        sum = sum + i;
    }
    return sum;
    // Should be 5 + 4 + 3 + 2 + 1 = 15
}

// run: test_for_loop_decrement() == 15

int test_for_loop_decrement_count() {
    int count = 0;
    for (int i = 10; i >= 1; i--) {
        count = count + 1;
    }
    return count;
    // Should be 10
}

// run: test_for_loop_decrement_count() == 10

int test_for_loop_decrement_range() {
    int sum = 0;
    for (int i = 10; i >= 5; i--) {
        sum = sum + i;
    }
    return sum;
    // Should be 10 + 9 + 8 + 7 + 6 + 5 = 45
}

// run: test_for_loop_decrement_range() == 45

int test_for_loop_decrement_zero() {
    int sum = 0;
    for (int i = 3; i >= 0; i--) {
        sum = sum + i;
    }
    return sum;
    // Should be 3 + 2 + 1 + 0 = 6
}

// run: test_for_loop_decrement_zero() == 6

