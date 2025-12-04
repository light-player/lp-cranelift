// test compile
// test run

int main() {
    float a = 2.5;
    float b = 1.5;
    if (a > b) {
        return 1;
    }
    return 0;
}

// CHECK: fcmp gt
// run: == 1

