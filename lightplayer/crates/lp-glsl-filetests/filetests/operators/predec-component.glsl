// test run
// target riscv32.fixed32

int main() {
    vec2 v = vec2(3.0, 4.0);
    float result = --v.y;  // v.y becomes 3.0, result is 3.0
    return int(result + v.x + v.y);  // Should be 3.0 + 3.0 + 3.0 = 9.0 -> 9
}

// run: main() == 9
