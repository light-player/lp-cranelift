// test error

double main() {  // ERROR: double not supported
    return 3.14;
}

// EXPECT_ERROR_CODE: E0109
// EXPECT_ERROR: type `Double` is not supported
// EXPECT_LOCATION: 2:8

