// test error

int main() {
    int x = 5;
    return y;  // ERROR: y is not defined
}

// EXPECT_ERROR_CODE: E0100
// EXPECT_ERROR: undefined variable `y`
// EXPECT_LOCATION: 4:12
// EXPECT_SPAN_TEXT:     return y;

