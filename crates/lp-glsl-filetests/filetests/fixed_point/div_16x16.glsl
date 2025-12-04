// test compile

float main() {
    float a = 10.0;
    float b = 4.0;
    return a / b;
}

// CHECK: iconst
// CHECK: sextend
// CHECK: ishl
// CHECK: sdiv
// CHECK: ireduce

