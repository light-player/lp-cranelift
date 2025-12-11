// test error

int main() {
    float x = fract(5);  // ERROR: fract requires float type
    return 0;
}

// EXPECT_ERROR_CODE: E0114
// EXPECT_ERROR: No matching overload for fract
// EXPECT_LOCATION: 4

