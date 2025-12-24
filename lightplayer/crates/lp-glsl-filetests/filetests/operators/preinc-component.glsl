// test run
// target riscv32.fixed32

int main() {
    vec2 v = vec2(1.0, 2.0);
    float result = ++v.x;  // v.x becomes 2.0, result is 2.0
    return int(result + v.x + v.y);  // Should be 2.0 + 2.0 + 2.0 = 6.0 -> 6
}

// run: main() == 6
