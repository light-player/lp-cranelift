// test error

int helper() {
    return 42;
}

// No main() function - should fail

// EXPECT_ERROR_CODE: E0108
// EXPECT_ERROR: no `main()` function found

