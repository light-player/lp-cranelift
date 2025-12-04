// test error

int main() {
    float x = mix(1.0, 2.0);  // ERROR: wrong argument count
    return 0;
}

// EXPECT_ERROR: No matching overload for mix

