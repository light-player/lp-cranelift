// test error

int main() {
    float x = sign();  // ERROR: wrong argument count
    return 0;
}

// EXPECT_ERROR: No matching overload for sign

