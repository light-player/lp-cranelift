// test error

int main() {
    int x = 5;
    if (x) {  // ERROR: condition must be bool
        return 1;
    }
    return 0;
}

// EXPECT_ERROR: Condition must be bool type

