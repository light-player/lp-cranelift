// test compile
// test run

int main() {
    int sum = 0;
    for (int i = 0; i < 5; i = i + 1) {
        sum = sum + i;
    }
    return sum;
}

// CHECK: brif
// CHECK: jump
// run: == 10


