// test run
// target riscv32.fixed32

// ============================================================================
// Continue in nested loops
// ============================================================================

int test_continue_nested_for_inner() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 5; j++) {
            if (j == 2) {
                continue;
            }
            sum = sum + 1;
        }
    }
    return sum;
    // Should be 3 * 4 = 12 (skips j=2 in inner loop)
}

// run: test_continue_nested_for_inner() == 12

int test_continue_nested_while_inner() {
    int sum = 0;
    int i = 0;
    while (i < 3) {
        int j = 0;
        while (j < 5) {
            if (j == 2) {
                j = j + 1;
                continue;
            }
            sum = sum + 1;
            j = j + 1;
        }
        i = i + 1;
    }
    return sum;
    // Should be 3 * 4 = 12 (skips j=2 in inner loop)
}

// run: test_continue_nested_while_inner() == 12

int test_continue_nested_mixed() {
    int sum = 0;
    for (int i = 0; i < 2; i++) {
        int j = 0;
        while (j < 4) {
            if (j == 1) {
                j = j + 1;
                continue;
            }
            sum = sum + 1;
            j = j + 1;
        }
    }
    return sum;
    // Should be 2 * 3 = 6 (skips j=1 in inner loop)
}

// run: test_continue_nested_mixed() == 6

int test_continue_nested_triple() {
    int count = 0;
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < 3; j++) {
            for (int k = 0; k < 4; k++) {
                if (k == 2) {
                    continue;
                }
                count = count + 1;
            }
        }
    }
    return count;
    // Should be 2 * 3 * 3 = 18 (skips k=2 in innermost loop)
}

// run: test_continue_nested_triple() == 18

