// test compile
// test run

int main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    vec3 c = a + b;  // (5.0, 7.0, 9.0)
    return 1;
}

// CHECK: fadd
// run: == 1

