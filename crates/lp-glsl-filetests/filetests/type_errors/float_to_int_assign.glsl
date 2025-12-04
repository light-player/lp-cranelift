// test error

int main() {
    int x = 3.14;  // ERROR: no implicit float → int
    return x;
}

// EXPECT_ERROR: Type mismatch in initialization: expected Int, got Float

