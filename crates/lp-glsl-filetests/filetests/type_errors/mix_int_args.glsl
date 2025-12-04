// test error

int main() {
    float x = mix(1, 2, 0);  // ERROR: mix requires float types
    return 0;
}

// EXPECT_ERROR: Verification error

