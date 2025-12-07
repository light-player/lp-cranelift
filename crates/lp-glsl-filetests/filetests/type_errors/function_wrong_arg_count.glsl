// test error

float add(float a, float b) {
    return a + b;
}

float main() {
    return add(1.0);  // ERROR: needs 2 args
}

// EXPECT_ERROR_CODE: E0114
// EXPECT_ERROR: no matching overload for function `add`
// EXPECT_LOCATION: 7:12

