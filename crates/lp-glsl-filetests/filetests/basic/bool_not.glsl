// test compile
// test run

bool main() {
    bool t = true;
    return !t;
}

// CHECK: iconst.i8 1
// CHECK: iconst.i8 0
// CHECK: icmp eq
// run: == false
