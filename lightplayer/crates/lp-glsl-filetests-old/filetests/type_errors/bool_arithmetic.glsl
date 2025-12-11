// test error

int main() {
    bool a = true;
    bool b = false;
    if (a + b) {  // ERROR: cannot add bools
        return 1;
    }
    return 0;
}

// EXPECT_ERROR_CODE: E0106
// EXPECT_ERROR: arithmetic operator Add requires numeric operands
// EXPECT_LOCATION: 5:9

