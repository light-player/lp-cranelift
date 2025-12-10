// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 v = vec3(-1.0, 0.5, 2.0);
    return clamp(v, 0.0, 1.0);  // (0.0, 0.5, 1.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v16 = iconst.i32 0x0001_0000
//     v17 = ineg v16  ; v16 = 0x0001_0000
//     v18 = iconst.i32 0x8000
//     v19 = iconst.i32 0x0002_0000
//     v20 = iconst.i32 0
//     v21 = iconst.i32 0x0001_0000
//     v22 = icmp sge v17, v20  ; v20 = 0
//     v23 = select v22, v17, v20  ; v20 = 0
//     v24 = icmp sge v18, v20  ; v18 = 0x8000, v20 = 0
//     v25 = select v24, v18, v20  ; v18 = 0x8000, v20 = 0
//     v26 = icmp sge v19, v20  ; v19 = 0x0002_0000, v20 = 0
//     v27 = select v26, v19, v20  ; v19 = 0x0002_0000, v20 = 0
//     v28 = icmp sle v23, v21  ; v21 = 0x0001_0000
//     v29 = select v28, v23, v21  ; v21 = 0x0001_0000
//     v30 = icmp sle v25, v21  ; v21 = 0x0001_0000
//     v31 = select v30, v25, v21  ; v21 = 0x0001_0000
//     v32 = icmp sle v27, v21  ; v21 = 0x0001_0000
//     v33 = select v32, v27, v21  ; v21 = 0x0001_0000
//     store notrap aligned v29, v0
//     store notrap aligned v31, v0+4
//     store notrap aligned v33, v0+8
//     return
//
// block1:
//     v34 = iconst.i32 0
//     store notrap aligned v34, v0  ; v34 = 0
//     v35 = iconst.i32 0
//     store notrap aligned v35, v0+4  ; v35 = 0
//     v36 = iconst.i32 0
//     store notrap aligned v36, v0+8  ; v36 = 0
//     return
// }
