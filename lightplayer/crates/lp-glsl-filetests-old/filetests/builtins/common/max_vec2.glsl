// test compile

// target riscv32.fixed32
vec2 main() {
    vec2 a = vec2(1.0, 5.0);
    vec2 b = vec2(3.0, 2.0);
    return max(a, b);  // (3.0, 5.0)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0005_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0002_0000
//     v5 = icmp sgt v1, v3  ; v1 = 0x0001_0000, v3 = 0x0003_0000
//     v6 = select v5, v1, v3  ; v1 = 0x0001_0000, v3 = 0x0003_0000
//     v7 = icmp sgt v2, v4  ; v2 = 0x0005_0000, v4 = 0x0002_0000
//     v8 = select v7, v2, v4  ; v2 = 0x0005_0000, v4 = 0x0002_0000
//     store notrap aligned v6, v0
//     store notrap aligned v8, v0+4
//     return
//
// block1:
//     v9 = iconst.i32 0
//     store notrap aligned v9, v0  ; v9 = 0
//     v10 = iconst.i32 0
//     store notrap aligned v10, v0+4  ; v10 = 0
//     return
// }
