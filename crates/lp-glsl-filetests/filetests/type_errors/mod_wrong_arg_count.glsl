// test error

int main() {
    float x = mod(5.0);  // ERROR: wrong argument count
    return 0;
}

// EXPECT_ERROR: No matching overload for mod

