// test error

int main() {
    float f = 5.0;
    float x = f.x;  // ERROR: scalars don't have components
    return 1;
}

// EXPECT_ERROR_CODE: E0112
// EXPECT_ERROR: component access on non-vector type: Float
// EXPECT_LOCATION: 4:15



