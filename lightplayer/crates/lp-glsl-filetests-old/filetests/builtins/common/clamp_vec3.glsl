// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 v = vec3(-1.0, 0.5, 2.0);
    return clamp(v, 0.0, 1.0);  // (0.0, 0.5, 1.0)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = ineg v1  ; v1 = 0x0001_0000
//     v3 = iconst.i32 0x8000
//     v4 = iconst.i32 0x0002_0000
//     v5 = iconst.i32 0
//     v6 = iconst.i32 0x0001_0000
//     v7 = icmp sgt v2, v5  ; v5 = 0
//     v8 = select v7, v2, v5  ; v5 = 0
//     v9 = icmp sgt v3, v5  ; v3 = 0x8000, v5 = 0
//     v10 = select v9, v3, v5  ; v3 = 0x8000, v5 = 0
//     v11 = icmp sgt v4, v5  ; v4 = 0x0002_0000, v5 = 0
//     v12 = select v11, v4, v5  ; v4 = 0x0002_0000, v5 = 0
//     v13 = icmp slt v8, v6  ; v6 = 0x0001_0000
//     v14 = select v13, v8, v6  ; v6 = 0x0001_0000
//     v15 = icmp slt v10, v6  ; v6 = 0x0001_0000
//     v16 = select v15, v10, v6  ; v6 = 0x0001_0000
//     v17 = icmp slt v12, v6  ; v6 = 0x0001_0000
//     v18 = select v17, v12, v6  ; v6 = 0x0001_0000
//     store notrap aligned v14, v0
//     store notrap aligned v16, v0+4
//     store notrap aligned v18, v0+8
//     return
//
// block1:
//     v19 = iconst.i32 0
//     store notrap aligned v19, v0  ; v19 = 0
//     v20 = iconst.i32 0
//     store notrap aligned v20, v0+4  ; v20 = 0
//     v21 = iconst.i32 0
//     store notrap aligned v21, v0+8  ; v21 = 0
//     return
// }
