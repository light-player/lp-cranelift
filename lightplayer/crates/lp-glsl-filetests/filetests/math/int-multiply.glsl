

// test run
// target riscv32.fixed32

int mul_int(int a, int b) {
    return a * b;
}

// #run: mul_int(0, 5) == 0
// #run: mul_int(1, 5) == 5
// #run: mul_int(2, 3) == 6
// #run: mul_int(-2, 3) == -6
// #run: mul_int(-2, -3) == 6
// #run: mul_int(100, 200) == 20000

ivec2 mul_ivec2(ivec2 a, ivec2 b) {
    return a * b;
}

// #run: mul_ivec2(ivec2(2, 3), ivec2(4, 5)) == ivec2(8, 15)
// #run: mul_ivec2(ivec2(-1, 2), ivec2(3, -4)) == ivec2(-3, -8)

ivec3 mul_ivec3(ivec3 a, ivec3 b) {
    return a * b;
}

// #run: mul_ivec3(ivec3(1, 2, 3), ivec3(4, 5, 6)) == ivec3(4, 10, 18)

ivec4 mul_ivec4(ivec4 a, ivec4 b) {
    return a * b;
}

// #run: mul_ivec4(ivec4(1, 2, 3, 4), ivec4(5, 6, 7, 8)) == ivec4(5, 12, 21, 32)

ivec2 mul_scalar_ivec2(int s, ivec2 v) {
    return s * v;
}

// #run: mul_scalar_ivec2(2, ivec2(1, 3)) == ivec2(2, 6)
// #run: mul_scalar_ivec2(-1, ivec2(2, 4)) == ivec2(-2, -4)
