// test error

int main() {
    int x = 5;
    bool y = true;
    if (x == y) {  // ERROR: cannot compare int and bool
        return 1;
    }
    return 0;
}

// EXPECT_ERROR_CODE: E0106
// EXPECT_ERROR: comparison operator Equal requires numeric operands
// EXPECT_LOCATION: 5:9

