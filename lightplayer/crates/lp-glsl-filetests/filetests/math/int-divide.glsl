

// test run
// target riscv32.fixed32

int divide_int(int a, int b) {
    return a / b;
}

// #run: divide_int(10, 2) == 5
// #run: divide_int(-10, 2) == -5
// #run: divide_int(-10, -2) == 5
// #run: divide_int(7, 2) == 3
// #run: divide_int(1000, 10) == 100

ivec2 divide_ivec2(ivec2 a, ivec2 b) {
    return a / b;
}

// #run: divide_ivec2(ivec2(10, 20), ivec2(2, 4)) == ivec2(5, 5)
// #run: divide_ivec2(ivec2(7, 15), ivec2(2, 3)) == ivec2(3, 5)

ivec3 divide_ivec3(ivec3 a, ivec3 b) {
    return a / b;
}

// #run: divide_ivec3(ivec3(10, 20, 30), ivec3(2, 4, 5)) == ivec3(5, 5, 6)

ivec4 divide_ivec4(ivec4 a, ivec4 b) {
    return a / b;
}

// #run: divide_ivec4(ivec4(10, 20, 30, 40), ivec4(2, 4, 5, 8)) == ivec4(5, 5, 6, 5)

ivec2 divide_scalar_ivec2(int s, ivec2 v) {
    return s / v;
}

// #run: divide_scalar_ivec2(10, ivec2(2, 5)) == ivec2(5, 2)
