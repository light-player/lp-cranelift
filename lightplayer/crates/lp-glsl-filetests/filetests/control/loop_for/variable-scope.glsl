// test run
// target riscv32.fixed32

// ============================================================================
// Variable scoping in for loops (init-expression scope)
// Spec: Variables declared in init-expression are only in scope until the end
//       of the sub-statement of the for loop
// ============================================================================

int test_for_loop_init_scope() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        sum = sum + i;
    }
    // i is out of scope here
    return sum;
    // Should be 0 + 1 + 2 = 3
}

// run: test_for_loop_init_scope() == 3

int test_for_loop_init_shadowing() {
    int i = 100;
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        sum = sum + i;
    }
    // Outer i should be unchanged
    return i;
    // Should be 100 (outer i)
}

// run: test_for_loop_init_shadowing() == 3

int test_for_loop_condition_scope() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        int j = i * 2;
        sum = sum + j;
    }
    return sum;
    // Should be 0 + 2 + 4 + 6 + 8 = 20
}

// run: test_for_loop_condition_scope() == 20

int test_for_loop_multiple_loops() {
    int sum = 0;
    for (int i = 0; i < 2; i++) {
        sum = sum + i;
    }
    for (int i = 0; i < 3; i++) {
        sum = sum + i;
    }
    return sum;
    // Should be (0 + 1) + (0 + 1 + 2) = 4
}

// run: test_for_loop_multiple_loops() == 4

int test_for_loop_init_scope_after_loop() {
    int x = 0;
    for (int i = 0; i < 2; i++) {
        x = x + i;
    }
    // i is out of scope - should not be accessible
    return x;
    // Should be 0 + 1 = 1
}

// run: test_for_loop_init_scope_after_loop() == 1

int test_for_loop_loop_expression_scope() {
    int sum = 0;
    int i = 0;
    for (i = 0; i < 3; i++) {
        sum = sum + i;
    }
    // i should still be in scope (not declared in init-expression)
    return i;
    // Should be 3 (value after loop)
}

// run: test_for_loop_loop_expression_scope() == 3

int test_for_loop_nested_same_name() {
    int sum = 0;
    for (int i = 0; i < 2; i++) {
        for (int i = 0; i < 3; i++) {
            sum = sum + 1;
        }
    }
    return sum;
    // Should be 2 * 3 = 6 (inner i shadows outer i)
}

// run: test_for_loop_nested_same_name() == 6

int test_for_loop_condition_declaration() {
    int sum = 0;
    for (int i = 0; int j = i < 3; i++) {
        sum = sum + i;
    }
    // Test condition-expression variable declaration
    return sum;
    // Should be 0 + 1 + 2 = 3
}

// run: test_for_loop_condition_declaration() == 3
