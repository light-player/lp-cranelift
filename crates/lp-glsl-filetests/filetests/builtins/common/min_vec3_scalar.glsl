// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 v = vec3(5.0, 2.0, 7.0);
    return min(v, 4.0);  // (4.0, 2.0, 4.0)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0005_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0007_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = icmp slt v1, v4  ; v1 = 0x0005_0000, v4 = 0x0004_0000
//     v6 = select v5, v1, v4  ; v1 = 0x0005_0000, v4 = 0x0004_0000
//     v7 = icmp slt v2, v4  ; v2 = 0x0002_0000, v4 = 0x0004_0000
//     v8 = select v7, v2, v4  ; v2 = 0x0002_0000, v4 = 0x0004_0000
//     v9 = icmp slt v3, v4  ; v3 = 0x0007_0000, v4 = 0x0004_0000
//     v10 = select v9, v3, v4  ; v3 = 0x0007_0000, v4 = 0x0004_0000
//     store notrap aligned v6, v0
//     store notrap aligned v8, v0+4
//     store notrap aligned v10, v0+8
//     return
//
// block1:
//     v11 = iconst.i32 0
//     store notrap aligned v11, v0  ; v11 = 0
//     v12 = iconst.i32 0
//     store notrap aligned v12, v0+4  ; v12 = 0
//     v13 = iconst.i32 0
//     store notrap aligned v13, v0+8  ; v13 = 0
//     return
// }
