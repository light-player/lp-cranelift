// test riscv32.fixed32
// test run

vec3 triple(vec3 v) {
    return v * 3.0;
}

vec3 main() {
    return triple(vec3(1.0, 2.0, 3.0));
}

// run: ≈ vec3(3.0, 6.0, 9.0)

