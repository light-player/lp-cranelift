// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = fract(3.75);  // 3.75 - floor(3.75) = 3.75 - 3.0 = 0.75
    return result > 0.74 && result < 0.76;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0x0003_c000
//     v1 = iconst.i32 16
//     v2 = sshr v0, v1  ; v0 = 0x0003_c000, v1 = 16
//     v3 = ishl v2, v1  ; v1 = 16
//     v4 = isub v0, v3  ; v0 = 0x0003_c000
//     v5 = iconst.i32 0xbd71
//     v6 = icmp sgt v4, v5  ; v5 = 0xbd71
//     v7 = sextend.i32 v6
//     v8 = iconst.i8 1
//     v9 = iconst.i8 0
//     v10 = select v7, v8, v9  ; v8 = 1, v9 = 0
//     v11 = iconst.i32 0xc28f
//     v12 = icmp slt v4, v11  ; v11 = 0xc28f
//     v13 = sextend.i32 v12
//     v14 = iconst.i8 1
//     v15 = iconst.i8 0
//     v16 = select v13, v14, v15  ; v14 = 1, v15 = 0
//     v17 = iconst.i8 0
//     v18 = iconst.i8 1
//     v19 = icmp ne v10, v17  ; v17 = 0
//     v20 = icmp ne v16, v17  ; v17 = 0
//     v21 = select v20, v18, v17  ; v18 = 1, v17 = 0
//     v22 = select v19, v21, v17  ; v17 = 0
//     return v22
//
// block1:
//     v23 = iconst.i8 0
//     return v23  ; v23 = 0
// }
// run: == true
