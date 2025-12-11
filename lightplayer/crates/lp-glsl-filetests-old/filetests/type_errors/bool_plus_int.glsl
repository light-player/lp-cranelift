// test error

int main() {
    int x = 5;
    bool y = true;
    return x + y;  // ERROR: cannot add int and bool
}

// EXPECT_ERROR_CODE: E0106
// EXPECT_ERROR: arithmetic operator Add requires numeric operands
// EXPECT_LOCATION: 5:12

