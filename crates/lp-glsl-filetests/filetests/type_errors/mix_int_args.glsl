// test error

int main() {
    float x = mix(1, 2, 0);  // ERROR: mix requires float types
    return 0;
}

// EXPECT_ERROR_CODE: E0114
// EXPECT_ERROR: No matching overload
// EXPECT_LOCATION: 4

