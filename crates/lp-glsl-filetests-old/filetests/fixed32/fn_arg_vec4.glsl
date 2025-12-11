// test riscv32.fixed32
// test run

vec4 quadruple(vec4 v) {
    return v * 4.0;
}

vec4 main() {
    return quadruple(vec4(1.0, 2.0, 3.0, 4.0));
}

// run: ≈ vec4(4.0, 8.0, 12.0, 16.0)


