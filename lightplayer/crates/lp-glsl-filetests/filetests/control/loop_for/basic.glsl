// test run
// target riscv32.fixed32

// ============================================================================
// Basic for loops with increment
// ============================================================================

int test_for_loop_basic() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum = sum + i;
    }
    return sum;
    // Should be 0 + 1 + 2 + 3 + 4 = 10
}

// run: test_for_loop_basic() == 10

int test_for_loop_count() {
    int count = 0;
    for (int i = 0; i < 10; i++) {
        count = count + 1;
    }
    return count;
    // Should be 10
}

// run: test_for_loop_count() == 10

int test_for_loop_accumulate() {
    int sum = 0;
    for (int i = 1; i <= 5; i++) {
        sum = sum + i;
    }
    return sum;
    // Should be 1 + 2 + 3 + 4 + 5 = 15
}

// run: test_for_loop_accumulate() == 15

int test_for_loop_zero_iterations() {
    int sum = 0;
    for (int i = 0; i < 0; i++) {
        sum = sum + 1;
    }
    return sum;
    // Should be 0 (loop doesn't execute)
}

// run: test_for_loop_zero_iterations() == 0

int test_for_loop_single_iteration() {
    int sum = 0;
    for (int i = 0; i < 1; i++) {
        sum = sum + 1;
    }
    return sum;
    // Should be 1
}

// run: test_for_loop_single_iteration() == 1

int test_for_loop_post_increment() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        sum = sum + i;
    }
    return sum;
    // Should be 0 + 1 + 2 = 3
}

// run: test_for_loop_post_increment() == 3

