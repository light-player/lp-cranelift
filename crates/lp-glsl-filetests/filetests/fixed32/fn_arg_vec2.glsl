// test riscv32.fixed32
// test run

vec2 double(vec2 v) {
    return v * 2.0;
}

vec2 main() {
    return double(vec2(1.0, 2.0));
}

// Generated CLIF
// function u0:0(f32, f32) -> f32, f32 system_v {
//     sig0 = (f32, f32) -> f32, f32 system_v
//     fn0 = colocated u0:0 sig0
// block0(v0: f32, v1: f32):
//     v2 = f32const 2.0
//     v3 = fmul v0, v2
//     v4 = fmul v1, v2
//     return v3, v4
// block1:
//     v5 = f32const 0.0
//     return v5, v5
// }
// function u0:1(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 1.0
//     v2 = f32const 2.0
//     v3 = call fn0(v1, v2)
//     store notrap aligned v3, v0
//     store notrap aligned v4, v0+4
//     return
// block1:
//     v5 = f32const 0.0
//     store notrap aligned v5, v0
//     store notrap aligned v5, v0+4
//     return
// }
//
// Transformed CLIF
// function u0:0(i32, i32) -> i32, i32 system_v {
//     sig0 = (i32, i32) -> i32, i32 system_v
//     fn0 = colocated u0:0 sig0
// block0(v0: i32, v1: i32):
//     v2 = iconst.i32 0x0002_0000
//     v3 = imul v0, v2
//     v4 = iconst.i32 16
//     v5 = sshr v3, v4
//     v6 = imul v1, v2
//     v7 = sshr v6, v4
//     return v5, v7
// block1:
//     v8 = iconst.i32 0
//     return v8, v8
// }
// function u0:1(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3, v4 = call fn0(v1, v2)
//     store notrap aligned v3, v0
//     store notrap aligned v4, v0+4
//     return
// block1:
//     v5 = iconst.i32 0
//     store notrap aligned v5, v0
//     store notrap aligned v5, v0+4
//     return
// }
// run: ≈ vec2(2.0, 4.0)
