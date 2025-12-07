// test error

int main() {
    int a = 5;
    int b = 10;
    (a + b) = 20;  // ERROR: cannot assign to expression
    return 1;
}

// EXPECT_ERROR_CODE: E0115
// EXPECT_ERROR: assignment lhs must be variable
// EXPECT_LOCATION: 5:6
// EXPECT_SPAN_TEXT: (a + b) = 20;

