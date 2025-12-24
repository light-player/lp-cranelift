// test compile
// test transform.fixed32
// test run
// target riscv32.fixed32

float mul_float(float a, float b) {
    return a * b;
}

// #run: mul_float(0.0, 5.0) ~= 0.0
// #run: mul_float(1.0, 5.0) ~= 5.0
// #run: mul_float(2.5, 3.0) ~= 7.5
// #run: mul_float(-2.0, 3.0) ~= -6.0
// #run: mul_float(-2.0, -3.0) ~= 6.0
// #run: mul_float(0.5, 0.5) ~= 0.25
// #run: mul_float(100.0, 200.0) ~= 20000.0

vec2 mul_vec2(vec2 a, vec2 b) {
    return a * b;
}

// #run: mul_vec2(vec2(2.0, 3.0), vec2(4.0, 5.0)) ~= vec2(8.0, 15.0)
// #run: mul_vec2(vec2(-1.0, 2.0), vec2(3.0, -4.0)) ~= vec2(-3.0, -8.0)

vec3 mul_vec3(vec3 a, vec3 b) {
    return a * b;
}

// #run: mul_vec3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0)) ~= vec3(4.0, 10.0, 18.0)

vec4 mul_vec4(vec4 a, vec4 b) {
    return a * b;
}

// #run: mul_vec4(vec4(1.0, 2.0, 3.0, 4.0), vec4(5.0, 6.0, 7.0, 8.0)) ~= vec4(5.0, 12.0, 21.0, 32.0)

vec2 mul_scalar_vec2(float s, vec2 v) {
    return s * v;
}

// #run: mul_scalar_vec2(2.0, vec2(1.0, 3.0)) ~= vec2(2.0, 6.0)
// #run: mul_scalar_vec2(-1.0, vec2(2.0, 4.0)) ~= vec2(-2.0, -4.0)

mat2 mul_mat2(mat2 a, mat2 b) {
    return a * b;
}

// #run: mul_mat2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), mat2(vec2(5.0, 6.0), vec2(7.0, 8.0))) ~= mat2(vec2(23.0, 34.0), vec2(31.0, 46.0))

vec2 mul_vec2_mat2(vec2 v, mat2 m) {
    return v * m;
}

// #run: mul_vec2_mat2(vec2(1.0, 2.0), mat2(vec2(3.0, 4.0), vec2(5.0, 6.0))) ~= vec2(11.0, 17.0)

vec2 mul_mat2_vec2(mat2 m, vec2 v) {
    return m * v;
}

// #run: mul_mat2_vec2(mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)), vec2(5.0, 6.0)) ~= vec2(23.0, 34.0)

mat3 mul_mat3(mat3 a, mat3 b) {
    return a * b;
}

// #run: mul_mat3(mat3(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)), mat3(vec3(2.0, 0.0, 0.0), vec3(0.0, 2.0, 0.0), vec3(0.0, 0.0, 2.0))) ~= mat3(vec3(2.0, 0.0, 0.0), vec3(0.0, 2.0, 0.0), vec3(0.0, 0.0, 2.0))

vec3 mul_vec3_mat3(vec3 v, mat3 m) {
    return v * m;
}

// #run: mul_vec3_mat3(vec3(1.0, 2.0, 3.0), mat3(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0))) ~= vec3(1.0, 2.0, 3.0)

vec3 mul_mat3_vec3(mat3 m, vec3 v) {
    return m * v;
}

// #run: mul_mat3_vec3(mat3(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)), vec3(1.0, 2.0, 3.0)) ~= vec3(1.0, 2.0, 3.0)
