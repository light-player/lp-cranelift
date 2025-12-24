// test run
// target riscv32.fixed32

int main() {
    int x = 5;
    int result = ++x;  // Should increment x to 6, then return 6
    return result;     // Should return 6
}

// run: main() == 6
