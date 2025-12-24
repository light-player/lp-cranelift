// test compile
// test transform.fixed32
// test run
// target riscv32.fixed32

float divide_float(float a, float b) {
    return a / b;
}

// #run: divide_float(5.0, 1.0) ~= 5.0
// #run: divide_float(10.0, 2.0) ~= 5.0
// #run: divide_float(-10.0, 2.0) ~= -5.0
// #run: divide_float(-10.0, -2.0) ~= 5.0
// #run: divide_float(1.0, 2.0) ~= 0.5
// #run: divide_float(0.1, 0.2) ~= 0.5
// #run: divide_float(1000.0, 10.0) ~= 100.0

vec2 divide_vec2(vec2 a, vec2 b) {
    return a / b;
}

// #run: divide_vec2(vec2(10.0, 20.0), vec2(2.0, 4.0)) ~= vec2(5.0, 5.0)
// #run: divide_vec2(vec2(1.0, 2.0), vec2(2.0, 4.0)) ~= vec2(0.5, 0.5)

vec3 divide_vec3(vec3 a, vec3 b) {
    return a / b;
}

// #run: divide_vec3(vec3(10.0, 20.0, 30.0), vec3(2.0, 4.0, 5.0)) ~= vec3(5.0, 5.0, 6.0)

vec4 divide_vec4(vec4 a, vec4 b) {
    return a / b;
}

// #run: divide_vec4(vec4(10.0, 20.0, 30.0, 40.0), vec4(2.0, 4.0, 5.0, 8.0)) ~= vec4(5.0, 5.0, 6.0, 5.0)

vec2 divide_scalar_vec2(float s, vec2 v) {
    return s / v;
}

// #run: divide_scalar_vec2(10.0, vec2(2.0, 5.0)) ~= vec2(5.0, 2.0)

mat2 divide_mat2_scalar(mat2 a, float s) {
    return a / s;
}

// #run: divide_mat2_scalar(mat2(vec2(10.0, 20.0), vec2(30.0, 40.0)), 2.0) ~= mat2(vec2(5.0, 10.0), vec2(15.0, 20.0))
// #run: divide_mat2_scalar(mat2(vec2(8.0, 12.0), vec2(16.0, 20.0)), 4.0) ~= mat2(vec2(2.0, 3.0), vec2(4.0, 5.0))

mat3 divide_mat3_scalar(mat3 a, float s) {
    return a / s;
}

// #run: divide_mat3_scalar(mat3(vec3(10.0, 20.0, 30.0), vec3(40.0, 50.0, 60.0), vec3(70.0, 80.0, 90.0)), 10.0) ~= mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0))

mat4 divide_mat4_scalar(mat4 a, float s) {
    return a / s;
}

// #run: divide_mat4_scalar(mat4(vec4(10.0, 20.0, 30.0, 40.0), vec4(50.0, 60.0, 70.0, 80.0), vec4(90.0, 100.0, 110.0, 120.0), vec4(130.0, 140.0, 150.0, 160.0)), 10.0) ~= mat4(vec4(1.0, 2.0, 3.0, 4.0), vec4(5.0, 6.0, 7.0, 8.0), vec4(9.0, 10.0, 11.0, 12.0), vec4(13.0, 14.0, 15.0, 16.0))
