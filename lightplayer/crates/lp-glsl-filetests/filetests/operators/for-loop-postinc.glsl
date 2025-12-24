// test run
// target riscv32.fixed32

int main() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum = sum + i;
    }
    return sum;
}

// run: main() == 10
