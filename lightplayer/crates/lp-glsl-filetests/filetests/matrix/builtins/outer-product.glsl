// test run
// target riscv32.fixed32

// ============================================================================
// Outer product: outerProduct(c, r)
// outerProduct(c, r)[col][row] == c[col] * r[row]
// GLSL spec: m[col][row] - first index is column, second is row
// ============================================================================

float test_outer_product_vec2() {
    vec2 c = vec2(1.0, 2.0);
    vec2 r = vec2(3.0, 4.0);
    mat2 m = outerProduct(c, r);
    // m[0][0] (col 0, row 0) = c[0] * r[0] = 1.0 * 3.0 = 3.0
    // m[0][1] (col 0, row 1) = c[0] * r[1] = 1.0 * 4.0 = 4.0
    // m[1][0] (col 1, row 0) = c[1] * r[0] = 2.0 * 3.0 = 6.0
    // m[1][1] (col 1, row 1) = c[1] * r[1] = 2.0 * 4.0 = 8.0
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 3.0 + 6.0 + 4.0 + 8.0 = 21.0
}

// run: test_outer_product_vec2() ~= 21.0

float test_outer_product_vec2_verify() {
    vec2 c = vec2(10.0, 20.0);
    vec2 r = vec2(30.0, 40.0);
    mat2 m = outerProduct(c, r);
    // Verify: m[1][1] should be c[1] * r[1] = 20.0 * 40.0 = 800.0
    return m[1][1];
    // Should be 800.0
}

// run: test_outer_product_vec2_verify() ~= 800.0

float test_outer_product_vec3() {
    vec3 c = vec3(1.0, 2.0, 3.0);
    vec3 r = vec3(4.0, 5.0, 6.0);
    mat3 m = outerProduct(c, r);
    // Sum diagonal: m[i][i] = c[i] * r[i]
    return m[0][0] + m[1][1] + m[2][2];
    // Should be 4.0 + 10.0 + 18.0 = 32.0
}

// run: test_outer_product_vec3() ~= 32.0

float test_outer_product_vec4() {
    vec4 c = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 r = vec4(5.0, 6.0, 7.0, 8.0);
    mat4 m = outerProduct(c, r);
    // Sum first column (col 0): m[0][row] = c[0] * r[row]
    return m[0][0] + m[0][1] + m[0][2] + m[0][3];
    // Should be 5.0 + 6.0 + 7.0 + 8.0 = 26.0
}

// run: test_outer_product_vec4() ~= 26.0

