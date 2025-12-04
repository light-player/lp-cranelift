// test compile
// test run

int main() {
    int a = 10;
    int b = 20;
    return a + b;
}

// CHECK: iconst.i32 10
// CHECK: iconst.i32 20
// CHECK: iadd
// run: == 30
