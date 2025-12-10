// test compile

// target riscv32.fixed32
float main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    v.y = 5.0;  // v = (1.0, 5.0, 3.0)
    return v.y;  // 5.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0001_0000
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 0x0003_0000
//     v3 = iconst.i32 0x0005_0000
//     return v3  ; v3 = 0x0005_0000
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
