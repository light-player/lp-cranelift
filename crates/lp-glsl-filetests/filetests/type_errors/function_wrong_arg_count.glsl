// test error

float add(float a, float b) {
    return a + b;
}

float main() {
    return add(1.0);  // ERROR: needs 2 args
}

// EXPECT_ERROR: No matching function

