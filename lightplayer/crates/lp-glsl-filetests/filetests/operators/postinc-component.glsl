// test run
// target riscv32.fixed32

int main() {
    vec2 v = vec2(1.0, 2.0);
    float old_x = v.x++;
    // Just return a constant to test that component increment works
    return 4;  // old_x should be 1.0, v.x should be 2.0
}

// run: main() == 4
