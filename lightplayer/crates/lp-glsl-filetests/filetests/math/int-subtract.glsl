// test compile
// test transform.fixed32
// test run
// target riscv32.fixed32

int subtract_int(int a, int b) {
    return a - b;
}

// #run: subtract_int(0, 0) == 0
// #run: subtract_int(5, 2) == 3
// #run: subtract_int(-5, -2) == -3
// #run: subtract_int(5, -2) == 7
// #run: subtract_int(2, 5) == -3
// #run: subtract_int(1000, 500) == 500

ivec2 subtract_ivec2(ivec2 a, ivec2 b) {
    return a - b;
}

// #run: subtract_ivec2(ivec2(0, 0), ivec2(0, 0)) == ivec2(0, 0)
// #run: subtract_ivec2(ivec2(5, 10), ivec2(2, 3)) == ivec2(3, 7)
// #run: subtract_ivec2(ivec2(-1, 2), ivec2(3, -4)) == ivec2(-4, 6)

ivec3 subtract_ivec3(ivec3 a, ivec3 b) {
    return a - b;
}

// #run: subtract_ivec3(ivec3(10, 20, 30), ivec3(1, 2, 3)) == ivec3(9, 18, 27)

ivec4 subtract_ivec4(ivec4 a, ivec4 b) {
    return a - b;
}

// #run: subtract_ivec4(ivec4(10, 20, 30, 40), ivec4(1, 2, 3, 4)) == ivec4(9, 18, 27, 36)

ivec2 subtract_scalar_ivec2(int s, ivec2 v) {
    return s - v;
}

// #run: subtract_scalar_ivec2(5, ivec2(2, 3)) == ivec2(3, 2)
// #run: subtract_scalar_ivec2(-1, ivec2(2, 4)) == ivec2(-3, -5)
