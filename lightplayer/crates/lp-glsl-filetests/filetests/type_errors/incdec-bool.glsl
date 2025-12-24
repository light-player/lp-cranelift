// test error
// target riscv32.fixed32

int main() {
    bool b = true;
    // This should fail - increment/decrement not allowed on bool
    b++;
    return 0;
}

// EXPECT_ERROR_CODE: E0112
// EXPECT_ERROR: post-increment requires numeric operand
// EXPECT_LOCATION: 4
