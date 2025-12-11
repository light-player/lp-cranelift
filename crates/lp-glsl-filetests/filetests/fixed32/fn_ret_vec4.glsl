// test riscv32.fixed32
// test run

vec4 get_vec() {
    return vec4(1.0, 2.0, 3.0, 4.0);
}

vec4 main() {
    return get_vec();
}

// run: ≈ vec4(1.0, 2.0, 3.0, 4.0)
