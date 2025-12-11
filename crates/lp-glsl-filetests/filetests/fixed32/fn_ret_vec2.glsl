// test riscv32.fixed32
// test run

vec2 get_vec() {
    return vec2(10.0, 20.0);
}

vec2 main() {
    return get_vec();
}

// run: ≈ vec2(10.0, 20.0)

