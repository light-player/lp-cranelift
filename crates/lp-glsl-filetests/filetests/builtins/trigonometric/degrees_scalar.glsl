// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    float pi = 3.141592654;
    return degrees(pi);  // Should be 180.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0003_243f
//     v1 = iconst.i32 0x0039_4bb8
//     v2 = sextend.i64 v0  ; v0 = 0x0003_243f
//     v3 = sextend.i64 v1  ; v1 = 0x0039_4bb8
//     v4 = imul v2, v3
//     v5 = iconst.i64 16
//     v6 = sshr v4, v5  ; v5 = 16
//     v7 = ireduce.i32 v6
//     return v7
//
// block1:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
// }
// run: ~= 180 
