// test error

int main() {
    return unknown_func(42);  // ERROR: function not defined
}

// EXPECT_ERROR_CODE: E0101
// EXPECT_ERROR: undefined function `unknown_func`
// EXPECT_LOCATION: 3


