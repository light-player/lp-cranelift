// test compile
// test run

int main() {
    int x = 5;
    if (x > 0) {
        x = 10;
    }
    return x;
}

// CHECK: icmp sgt
// CHECK: brif
// run: == 10

