// test error

int main() {
    float x = step(1.0);  // ERROR: wrong argument count
    return 0;
}

// EXPECT_ERROR_CODE: E0114
// EXPECT_ERROR: No matching overload for step
// EXPECT_LOCATION: 4

