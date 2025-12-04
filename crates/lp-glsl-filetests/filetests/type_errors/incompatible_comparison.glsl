// test error

int main() {
    int x = 5;
    bool y = true;
    if (x == y) {  // ERROR: cannot compare int and bool
        return 1;
    }
    return 0;
}

// EXPECT_ERROR: Comparison.*requires.*numeric operands

