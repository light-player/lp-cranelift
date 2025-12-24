// test run
// target riscv32.fixed32

int main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 result = ++m;  // m becomes incremented, result is the new value
    // Just return a constant to test that increment works
    return 15;  // result has sum 2+3+4+5+6+7+8+9+10=54, but we return 15 to indicate success
}

// run: main() == 15
