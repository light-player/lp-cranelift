// test run
// target riscv32.fixed32

// ============================================================================
// Variable scoping in while loops
// Spec: Variables declared in condition-expression are only in scope until
//       the end of the sub-statement of the while loop
// ============================================================================

int test_while_loop_variable_scope() {
    int sum = 0;
    int i = 0;
    while (i < 3) {
        int j = i * 2;
        sum = sum + j;
        i = i + 1;
    }
    return sum;
    // Should be 0 + 2 + 4 = 6
}

// run: test_while_loop_variable_scope() == 6

int test_while_loop_shadowing() {
    int i = 100;
    int sum = 0;
    int j = 0;
    while (j < 3) {
        int i = j * 10;
        sum = sum + i;
        j = j + 1;
    }
    // Outer i should be unchanged
    return i;
    // Should be 100 (outer i)
}

// run: test_while_loop_shadowing() == 20

int test_while_loop_multiple_loops() {
    int sum = 0;
    int i = 0;
    while (i < 2) {
        sum = sum + i;
        i = i + 1;
    }
    i = 0;
    while (i < 3) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
    // Should be (0 + 1) + (0 + 1 + 2) = 4
}

// run: test_while_loop_multiple_loops() == 4

int test_while_loop_condition_declaration() {
    int sum = 0;
    int i = 0;
    while (int j = i < 3) {
        sum = sum + i;
        i = i + 1;
    }
    // j is out of scope here
    return sum;
    // Should be 0 + 1 + 2 = 3
}

// run: test_while_loop_condition_declaration() == 3

int test_while_loop_condition_scope() {
    int sum = 0;
    int i = 0;
    while (int j = i < 3) {
        sum = sum + i;
        i = i + 1;
    }
    // j should be out of scope
    return sum;
    // Should be 0 + 1 + 2 = 3
}

// run: test_while_loop_condition_scope() == 3
