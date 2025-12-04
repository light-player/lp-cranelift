// test compile

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// CHECK: iconst
// CHECK: sextend
// CHECK: imul
// CHECK: sshr
// CHECK: ireduce

