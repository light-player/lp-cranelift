// test run
// target riscv32.fixed32

int main() {
    int x = 8;
    int result = --x;  // Should decrement x to 7, then return 7
    return result;     // Should return 7
}

// run: main() == 7
