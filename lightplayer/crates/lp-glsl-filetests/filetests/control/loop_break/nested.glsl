// test run
// target riscv32.fixed32

// ============================================================================
// Break in nested loops (breaks inner loop)
// ============================================================================

int test_break_nested_for_inner() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 5; j++) {
            if (j >= 2) {
                break;
            }
            sum = sum + 1;
        }
    }
    return sum;
    // Should be 3 * 2 = 6 (breaks inner loop only)
}

// run: test_break_nested_for_inner() == 6

int test_break_nested_while_inner() {
    int sum = 0;
    int i = 0;
    while (i < 3) {
        int j = 0;
        while (j < 5) {
            if (j >= 2) {
                break;
            }
            sum = sum + 1;
            j = j + 1;
        }
        i = i + 1;
    }
    return sum;
    // Should be 3 * 2 = 6 (breaks inner loop only)
}

// run: test_break_nested_while_inner() == 6

int test_break_nested_mixed() {
    int sum = 0;
    for (int i = 0; i < 2; i++) {
        int j = 0;
        while (j < 4) {
            if (j >= 2) {
                break;
            }
            sum = sum + 1;
            j = j + 1;
        }
    }
    return sum;
    // Should be 2 * 2 = 4 (breaks inner loop only)
}

// run: test_break_nested_mixed() == 4

int test_break_nested_triple() {
    int count = 0;
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < 3; j++) {
            for (int k = 0; k < 4; k++) {
                if (k >= 2) {
                    break;
                }
                count = count + 1;
            }
        }
    }
    return count;
    // Should be 2 * 3 * 2 = 12 (breaks innermost loop only)
}

// run: test_break_nested_triple() == 12

