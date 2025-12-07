// test error

int main() {
    int x;
    bool y = true;
    x = y;  // ERROR: cannot assign bool to int
    return x;
}

// EXPECT_ERROR_CODE: E0102
// EXPECT_ERROR: type mismatch in assignment
// EXPECT_LOCATION: 5:9

