

// test run
// target riscv32.fixed32

// ============================================================================
// Scalar Operations
// ============================================================================

int add_int(int a, int b) {
    return a + b;
}

// #run: add_int(0, 0) == 0
// #run: add_int(1, 2) == 3
// #run: add_int(-1, -2) == -3
// #run: add_int(5, -3) == 2
// #run: add_int(-5, 3) == -2
// #run: add_int(1000, 2000) == 3000
// #run: add_int(2147483647, 0) == 2147483647
// #run: add_int(-2147483648, 0) == -2147483648

// ============================================================================
// Vector Operations
// ============================================================================

ivec2 add_ivec2(ivec2 a, ivec2 b) {
    return a + b;
}

// #run: add_ivec2(ivec2(0, 0), ivec2(0, 0)) == ivec2(0, 0)
// #run: add_ivec2(ivec2(1, 2), ivec2(3, 4)) == ivec2(4, 6)
// #run: add_ivec2(ivec2(-1, 2), ivec2(3, -4)) == ivec2(2, -2)
// #run: add_ivec2(ivec2(1000, 2000), ivec2(500, 750)) == ivec2(1500, 2750)

ivec3 add_ivec3(ivec3 a, ivec3 b) {
    return a + b;
}

// #run: add_ivec3(ivec3(0, 0, 0), ivec3(0, 0, 0)) == ivec3(0, 0, 0)
// #run: add_ivec3(ivec3(1, 2, 3), ivec3(4, 5, 6)) == ivec3(5, 7, 9)
// #run: add_ivec3(ivec3(-1, 2, -3), ivec3(4, -5, 6)) == ivec3(3, -3, 3)

ivec4 add_ivec4(ivec4 a, ivec4 b) {
    return a + b;
}

// #run: add_ivec4(ivec4(0, 0, 0, 0), ivec4(0, 0, 0, 0)) == ivec4(0, 0, 0, 0)
// #run: add_ivec4(ivec4(1, 2, 3, 4), ivec4(5, 6, 7, 8)) == ivec4(6, 8, 10, 12)
// #run: add_ivec4(ivec4(-1, 2, -3, 4), ivec4(5, -6, 7, -8)) == ivec4(4, -4, 4, -4)

// ============================================================================
// Scalar + Vector Operations
// ============================================================================

ivec2 add_scalar_ivec2(int s, ivec2 v) {
    return s + v;
}

// #run: add_scalar_ivec2(0, ivec2(0, 0)) == ivec2(0, 0)
// #run: add_scalar_ivec2(2, ivec2(1, 3)) == ivec2(3, 5)
// #run: add_scalar_ivec2(-1, ivec2(2, 4)) == ivec2(1, 3)
