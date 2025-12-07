// test error

int main() {
    int x = 3.14;  // ERROR: no implicit float → int
    return x;
}

// EXPECT_ERROR_CODE: E0102
// EXPECT_ERROR: type mismatch in assignment
// EXPECT_LOCATION: 3:13

