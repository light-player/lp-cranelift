// test run
// target riscv32.fixed32

mat4 add(mat4 a, mat4 b) {
    return a + b;
}

float test() {
    mat4 c = add(
        mat4(
            vec4(1.0, 2.0, 3.0, 4.0),
            vec4(5.0, 6.0, 7.0, 8.0), 
            vec4(9.0, 10.0, 11.0, 12.0), 
            vec4(13.0, 14.0, 15.0, 16.0)
        ), 
        mat4(
            vec4(17.0, 18.0, 19.0, 20.0), 
            vec4(21.0, 22.0, 23.0, 24.0),
            vec4(25.0, 26.0, 27.0, 28.0),
            vec4(29.0, 30.0, 31.0, 32.0)
        )
    );
    return c[0][0] + 1;
}

// #run: test() ~= 19
