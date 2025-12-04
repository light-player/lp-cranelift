// test compile
// test run

int main() {
    int x = 5;
    int result;
    if (x > 10) {
        result = 1;
    } else {
        result = 0;
    }
    return result;
}

// CHECK: brif
// run: == 0

