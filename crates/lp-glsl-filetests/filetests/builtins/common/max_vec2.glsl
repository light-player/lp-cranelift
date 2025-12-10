// test compile

// target riscv32.fixed32
vec2 main() {
    vec2 a = vec2(1.0, 5.0);
    vec2 b = vec2(3.0, 2.0);
    return max(a, b);  // (3.0, 5.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v9 = iconst.i32 0x0001_0000
//     v10 = iconst.i32 0x0005_0000
//     v11 = iconst.i32 0x0003_0000
//     v12 = iconst.i32 0x0002_0000
//     v13 = icmp sge v9, v11  ; v9 = 0x0001_0000, v11 = 0x0003_0000
//     v14 = select v13, v9, v11  ; v9 = 0x0001_0000, v11 = 0x0003_0000
//     v15 = icmp sge v10, v12  ; v10 = 0x0005_0000, v12 = 0x0002_0000
//     v16 = select v15, v10, v12  ; v10 = 0x0005_0000, v12 = 0x0002_0000
//     store notrap aligned v14, v0
//     store notrap aligned v16, v0+4
//     return
//
// block1:
//     v17 = iconst.i32 0
//     store notrap aligned v17, v0  ; v17 = 0
//     v18 = iconst.i32 0
//     store notrap aligned v18, v0+4  ; v18 = 0
//     return
// }
