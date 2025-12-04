// test error

int main() {
    int x;
    bool y = true;
    x = y;  // ERROR: cannot assign bool to int
    return x;
}

// EXPECT_ERROR: Type mismatch: cannot assign Bool to Int

