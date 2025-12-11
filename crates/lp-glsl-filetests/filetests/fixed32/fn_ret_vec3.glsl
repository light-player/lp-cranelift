// test riscv32.fixed32
// test run

vec3 get_vec() {
    return vec3(1.0, 2.0, 3.0);
}

vec3 main() {
    return get_vec();
}

// run: ≈ vec3(1.0, 2.0, 3.0)

