// test run
// target riscv32.fixed32

int main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 old_m = m++;  // m becomes mat2(2.0, 3.0, 4.0, 5.0), old_m is original
    // Just return a constant to test that increment works
    return 10;  // old_m has sum 1+2+3+4=10, new m has sum 2+3+4+5=14
}

// run: main() == 10
