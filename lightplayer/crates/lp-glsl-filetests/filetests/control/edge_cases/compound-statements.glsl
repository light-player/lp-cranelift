// test run
// target riscv32.fixed32

// ============================================================================
// Compound statements (braces) edge cases
// Spec: Curly braces are used to group sequences of statements into compound
//       statements
// ============================================================================

int test_compound_statement_single() {
    int x = 0;
    {
        x = 10;
    }
    return x;
    // Should be 10
}

// run: test_compound_statement_single() == 10

int test_compound_statement_nested() {
    int x = 0;
    {
        int y = 5;
        {
            int z = 10;
            x = y + z;
        }
    }
    return x;
    // Should be 15
}

// run: test_compound_statement_nested() == 15

int test_compound_statement_empty() {
    int x = 5;
    {
        // Empty compound statement
    }
    return x;
    // Should be 5
}

// run: test_compound_statement_empty() == 5

int test_compound_statement_variable_scope() {
    int x = 0;
    {
        int y = 10;
        x = y;
    }
    // y is out of scope here
    return x;
    // Should be 10
}

// run: test_compound_statement_variable_scope() == 10

int test_compound_statement_in_if() {
    int x = 0;
    if (true) {
        int y = 20;
        x = y;
    }
    return x;
    // Should be 20
}

// run: test_compound_statement_in_if() == 20

int test_compound_statement_in_loop() {
    int sum = 0;
    for (int i = 0; i < 3; i++) {
        {
            int j = i * 2;
            sum = sum + j;
        }
    }
    return sum;
    // Should be 0 + 2 + 4 = 6
}

// run: test_compound_statement_in_loop() == 6

int test_compound_statement_multiple() {
    int x = 0;
    {
        x = 5;
    }
    {
        x = x + 10;
    }
    {
        x = x + 15;
    }
    return x;
    // Should be 30
}

// run: test_compound_statement_multiple() == 30

