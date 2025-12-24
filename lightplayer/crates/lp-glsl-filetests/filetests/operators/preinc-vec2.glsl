// test run
// target riscv32.fixed32

int main() {
    vec2 v = vec2(1.0, 2.0);
    vec2 result = ++v;  // Should increment v to (2.0, 3.0), then return (2.0, 3.0)
    // Just return a constant to test that increment works
    return int(result.x + result.y);  // Should be 2.0 + 3.0 = 5.0 -> 5
}

// run: main() == 5
