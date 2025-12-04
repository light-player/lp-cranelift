// test compile
// test run

int main() {
    int sum = 0;
    int i = 0;
    while (i < 5) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

// CHECK: brif
// CHECK: jump
// run: == 10

