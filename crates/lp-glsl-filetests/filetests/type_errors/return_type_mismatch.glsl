// test error

int main() {
    return true;  // ERROR: returning bool from int function
}

// EXPECT_ERROR_CODE: E0116
// EXPECT_ERROR: return type mismatch: expected `Int`, found `Bool`
// EXPECT_LOCATION: 3:12

