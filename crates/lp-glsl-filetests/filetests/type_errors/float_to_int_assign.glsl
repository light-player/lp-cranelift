// test error

int main() {
    int x = 3.14;  // ERROR: no implicit float → int
    return x;
}

// EXPECT_ERROR: cannot assign.*Float.*to.*Int

