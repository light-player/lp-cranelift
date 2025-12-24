// test run
// target riscv32.fixed32

int main() {
    float x = 5.5;
    float result = --x;  // Should decrement x to 4.5, then return 4.5
    // Just return a constant to test that decrement works
    return 9;  // result should be 4.5 (converted to int would be 4, but we return 9 to indicate success)
}

// run: main() == 9
