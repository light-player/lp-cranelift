// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return ceil(3.2);  // Should return 4.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v3 = iconst.i32 0x0003_3333
//     v4 = iconst.i64 16
//     v5 = sextend.i64 v3  ; v3 = 0x0003_3333
//     v6 = iconst.i64 0xffff
//     v7 = band v5, v6  ; v6 = 0xffff
//     v8 = icmp_imm ne v7, 0
//     v9 = sshr v5, v4  ; v4 = 16
//     v10 = iconst.i64 1
//     v11 = iadd v9, v10  ; v10 = 1
//     v12 = select v8, v11, v9
//     v13 = ishl v12, v4  ; v4 = 16
//     v14 = ireduce.i32 v13
//     return v14
//
// block1:
//     v15 = iconst.i32 0
//     return v15  ; v15 = 0
// }
// run: ~= 4.0
