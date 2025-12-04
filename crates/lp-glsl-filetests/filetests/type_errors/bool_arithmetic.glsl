// test error

int main() {
    bool a = true;
    bool b = false;
    if (a + b) {  // ERROR: cannot add bools
        return 1;
    }
    return 0;
}

// EXPECT_ERROR: Arithmetic operator Add requires numeric operands, got Bool and Bool

