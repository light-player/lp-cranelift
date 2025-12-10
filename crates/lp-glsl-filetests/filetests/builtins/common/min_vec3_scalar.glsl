// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 v = vec3(5.0, 2.0, 7.0);
    return min(v, 4.0);  // (4.0, 2.0, 4.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v11 = iconst.i32 0x0005_0000
//     v12 = iconst.i32 0x0002_0000
//     v13 = iconst.i32 0x0007_0000
//     v14 = iconst.i32 0x0004_0000
//     v15 = icmp sle v11, v14  ; v11 = 0x0005_0000, v14 = 0x0004_0000
//     v16 = select v15, v11, v14  ; v11 = 0x0005_0000, v14 = 0x0004_0000
//     v17 = icmp sle v12, v14  ; v12 = 0x0002_0000, v14 = 0x0004_0000
//     v18 = select v17, v12, v14  ; v12 = 0x0002_0000, v14 = 0x0004_0000
//     v19 = icmp sle v13, v14  ; v13 = 0x0007_0000, v14 = 0x0004_0000
//     v20 = select v19, v13, v14  ; v13 = 0x0007_0000, v14 = 0x0004_0000
//     store notrap aligned v16, v0
//     store notrap aligned v18, v0+4
//     store notrap aligned v20, v0+8
//     return
//
// block1:
//     v21 = iconst.i32 0
//     store notrap aligned v21, v0  ; v21 = 0
//     v22 = iconst.i32 0
//     store notrap aligned v22, v0+4  ; v22 = 0
//     v23 = iconst.i32 0
//     store notrap aligned v23, v0+8  ; v23 = 0
//     return
// }
