// test compile
// test run
// target riscv32

vec3 main() {
    return ceil(vec3(3.2, -2.7, 0.1));  // Should return (4.0, -2.0, 1.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.99999ap1
//     v2 = f32const 0x1.59999ap1
//     v3 = fneg v2  ; v2 = 0x1.59999ap1
//     v4 = f32const 0x1.99999ap-4
//     v5 = ceil v1  ; v1 = 0x1.99999ap1
//     v6 = ceil v3
//     v7 = ceil v4  ; v4 = 0x1.99999ap-4
//     store notrap aligned v5, v0
//     store notrap aligned v6, v0+4
//     store notrap aligned v7, v0+8
//     return
//
// block1:
//     v8 = f32const 0.0
//     store notrap aligned v8, v0  ; v8 = 0.0
//     v9 = f32const 0.0
//     store notrap aligned v9, v0+4  ; v9 = 0.0
//     v10 = f32const 0.0
//     store notrap aligned v10, v0+8  ; v10 = 0.0
//     return
// }
// run: ≈ vec3(4, -2, 1) (tolerance: 0.01)
