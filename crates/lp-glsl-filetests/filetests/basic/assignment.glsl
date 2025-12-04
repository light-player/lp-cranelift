// test compile
// test run

int main() {
    int x = 5;
    int y = x + 10;
    int z = y * 2;
    return z;
}

// CHECK: iconst.i32 5
// CHECK: iconst.i32 10
// CHECK: iadd
// run: == 30
