// test riscv32.fixed32
// test run

vec2 scale(vec2 v) {
    return v * 2.5;
}

vec2 main() {
    return scale(vec2(2.0, 4.0));
}

// run: ≈ vec2(5.0, 10.0)


