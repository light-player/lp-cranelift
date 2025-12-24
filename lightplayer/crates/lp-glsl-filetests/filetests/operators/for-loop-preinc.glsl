// test run
// target riscv32.fixed32

int main() {
    int sum = 0;
    for (int i = 0; ++i < 5; ) {
        sum = sum + i;
    }
    return sum;  // i goes: 1,2,3,4 (when ++i < 5 fails)
                 // sum = 0+1+2+3+4 = 10
}

// run: main() == 10
