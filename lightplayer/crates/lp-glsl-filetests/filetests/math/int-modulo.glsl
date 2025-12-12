

// test run
// target riscv32.fixed32

int mod_int(int a, int b) {
    return a % b;
}

// #run: mod_int(10, 3) == 1
// #run: mod_int(10, 5) == 0
// #run: mod_int(3, 2) == 1
// #run: mod_int(100, 7) == 2
// #run: mod_int(15, 4) == 3

ivec2 mod_ivec2(ivec2 a, ivec2 b) {
    return a % b;
}

// #run: mod_ivec2(ivec2(10, 20), ivec2(3, 7)) == ivec2(1, 6)
// #run: mod_ivec2(ivec2(15, 25), ivec2(4, 5)) == ivec2(3, 0)

ivec3 mod_ivec3(ivec3 a, ivec3 b) {
    return a % b;
}

// #run: mod_ivec3(ivec3(10, 20, 30), ivec3(3, 7, 11)) == ivec3(1, 6, 8)

ivec4 mod_ivec4(ivec4 a, ivec4 b) {
    return a % b;
}

// #run: mod_ivec4(ivec4(10, 20, 30, 40), ivec4(3, 7, 11, 13)) == ivec4(1, 6, 8, 1)

ivec2 mod_scalar_ivec2(int s, ivec2 v) {
    return s % v;
}

// #run: mod_scalar_ivec2(10, ivec2(3, 7)) == ivec2(1, 3)
