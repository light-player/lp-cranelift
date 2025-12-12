

// test run
// target riscv32.fixed32

float subtract_float(float a, float b) {
    return a - b;
}

// #run: subtract_float(0.0, 0.0) ~= 0.0
// #run: subtract_float(5.0, 2.0) ~= 3.0
// #run: subtract_float(-5.0, -2.0) ~= -3.0
// #run: subtract_float(5.0, -2.0) ~= 7.0
// #run: subtract_float(2.0, 5.0) ~= -3.0
// #run: subtract_float(1.0001, 1.0) ~= 0.0001
// #run: subtract_float(100.0, 50.0) ~= 50.0

vec2 subtract_vec2(vec2 a, vec2 b) {
    return a - b;
}

// #run: subtract_vec2(vec2(0.0, 0.0), vec2(0.0, 0.0)) ~= vec2(0.0, 0.0)
// #run: subtract_vec2(vec2(5.0, 10.0), vec2(2.0, 3.0)) ~= vec2(3.0, 7.0)
// #run: subtract_vec2(vec2(-1.0, 2.0), vec2(3.0, -4.0)) ~= vec2(-4.0, 6.0)

vec3 subtract_vec3(vec3 a, vec3 b) {
    return a - b;
}

// #run: subtract_vec3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0)) ~= vec3(-3.0, -3.0, -3.0)

vec4 subtract_vec4(vec4 a, vec4 b) {
    return a - b;
}

// #run: subtract_vec4(vec4(10.0, 20.0, 30.0, 40.0), vec4(1.0, 2.0, 3.0, 4.0)) ~= vec4(9.0, 18.0, 27.0, 36.0)

vec2 subtract_scalar_vec2(float s, vec2 v) {
    return s - v;
}

// #run: subtract_scalar_vec2(5.0, vec2(2.0, 3.0)) ~= vec2(3.0, 2.0)
// #run: subtract_scalar_vec2(-1.0, vec2(2.0, 4.0)) ~= vec2(-3.0, -5.0)

mat2 subtract_mat2(mat2 a, mat2 b) {
    return a - b;
}

// #run: subtract_mat2(mat2(vec2(5.0, 10.0), vec2(15.0, 20.0)), mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))) ~= mat2(vec2(4.0, 8.0), vec2(12.0, 16.0))

mat3 subtract_mat3(mat3 a, mat3 b) {
    return a - b;
}

// #run: subtract_mat3(mat3(vec3(10.0, 20.0, 30.0), vec3(40.0, 50.0, 60.0), vec3(70.0, 80.0, 90.0)), mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0))) ~= mat3(vec3(9.0, 18.0, 27.0), vec3(36.0, 45.0, 54.0), vec3(63.0, 72.0, 81.0))

mat4 subtract_mat4(mat4 a, mat4 b) {
    return a - b;
}

// #run: subtract_mat4(mat4(vec4(10.0, 20.0, 30.0, 40.0), vec4(50.0, 60.0, 70.0, 80.0), vec4(90.0, 100.0, 110.0, 120.0), vec4(130.0, 140.0, 150.0, 160.0)), mat4(vec4(1.0, 2.0, 3.0, 4.0), vec4(5.0, 6.0, 7.0, 8.0), vec4(9.0, 10.0, 11.0, 12.0), vec4(13.0, 14.0, 15.0, 16.0))) ~= mat4(vec4(9.0, 18.0, 27.0, 36.0), vec4(45.0, 54.0, 63.0, 72.0), vec4(81.0, 90.0, 99.0, 108.0), vec4(117.0, 126.0, 135.0, 144.0))
