// test error

int main() {
    return true;  // ERROR: returning bool from int function
}

// EXPECT_ERROR_CODE: E0400
// EXPECT_ERROR: code generation failed: Compilation error: Verifier errors
// EXPECT_LOCATION: 3:12

