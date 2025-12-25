// test run
// target riscv32.fixed32

// ============================================================================
// outerProduct(): Outer product function
// outerProduct(vec, vec) - outer product of two vectors
// Returns matrix: column vector * row vector
// ============================================================================

mat2 test_outerproduct_vec2() {
    // outerProduct(vec2, vec2) returns mat2
    vec2 a = vec2(1.0, 2.0);
    vec2 b = vec2(3.0, 4.0);
    return outerProduct(a, b);
}

// run: test_outerproduct_vec2() ~= mat2(3.0, 4.0, 6.0, 8.0)

mat3 test_outerproduct_vec3() {
    // outerProduct(vec3, vec3) returns mat3
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(2.0, 2.0, 2.0);
    return outerProduct(a, b);
}

// run: test_outerproduct_vec3() ~= mat3(2.0, 2.0, 2.0, 4.0, 4.0, 4.0, 6.0, 6.0, 6.0)

mat4 test_outerproduct_vec4() {
    // outerProduct(vec4, vec4) returns mat4
    vec4 a = vec4(1.0, 1.0, 1.0, 1.0);
    vec4 b = vec4(1.0, 2.0, 3.0, 4.0);
    return outerProduct(a, b);
}

// run: test_outerproduct_vec4() ~= mat4(1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0)

mat2x3 test_outerproduct_vec2_vec3() {
    // outerProduct(vec2, vec3) returns mat2x3
    vec2 a = vec2(2.0, 3.0);
    vec3 b = vec3(1.0, 2.0, 3.0);
    return outerProduct(a, b);
}

// run: test_outerproduct_vec2_vec3() ~= mat2x3(2.0, 4.0, 6.0, 3.0, 6.0, 9.0)

mat3x2 test_outerproduct_vec3_vec2() {
    // outerProduct(vec3, vec2) returns mat3x2
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec2 b = vec2(2.0, 3.0);
    return outerProduct(a, b);
}

// run: test_outerproduct_vec3_vec2() ~= mat3x2(2.0, 3.0, 4.0, 6.0, 6.0, 9.0)
