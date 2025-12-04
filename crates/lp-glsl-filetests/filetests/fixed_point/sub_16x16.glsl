// test compile
// test fixed16

float main() {
    float a = 5.5;
    float b = 2.25;
    return a - b;
}

// CHECK: iconst
// CHECK: isub
// CHECK-NOT: fsub

