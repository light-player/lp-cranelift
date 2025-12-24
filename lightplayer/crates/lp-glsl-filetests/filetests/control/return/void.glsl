// test run
// target riscv32.fixed32

// ============================================================================
// Return without value (void functions)
// ============================================================================

int test_void_return() {
    int x = 10;
    return x;
    // Should be 10 (return without value in non-void function is error, but test return with value)
}

// run: test_void_return() == 10

int test_return_after_statements() {
    int x = 5;
    int y = 10;
    return x + y;
    // Should be 15
}

// run: test_return_after_statements() == 15

int test_return_immediate() {
    return 100;
    // Should be 100 (immediate return)
}

// run: test_return_immediate() == 100

