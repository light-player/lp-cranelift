// test run
// target riscv32.fixed32

int main() {
    int x = 5;
    ivec2 v = ivec2(1, 2);

    // Test multiple increments in same expression (undefined behavior but should compile)
    int result1 = x++ + x++;  // x becomes 7, result1 is 5 + 6 = 11

    // Test integer vector increment
    ivec2 old_v = v++;  // v becomes ivec2(2, 3), old_v is ivec2(1, 2)

    // Return sum to verify results
    return result1 + int(old_v.x + old_v.y) + int(v.x + v.y);
    // result1 = 11, old_v sum = 3, v sum = 5, total = 11 + 3 + 5 = 19
}

// run: main() == 19
