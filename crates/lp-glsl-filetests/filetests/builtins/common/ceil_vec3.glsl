// test compile
// test run
// target riscv32.fixed32

vec3 main() {
    return ceil(vec3(3.2, -2.7, 0.1));  // Should return (4.0, -2.0, 1.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v11 = iconst.i32 0x0003_3333
//     v12 = iconst.i32 0x0002_b333
//     v13 = ineg v12  ; v12 = 0x0002_b333
//     v14 = iconst.i32 6554
//     v15 = iconst.i64 16
//     v16 = sextend.i64 v11  ; v11 = 0x0003_3333
//     v17 = iconst.i64 0xffff
//     v18 = band v16, v17  ; v17 = 0xffff
//     v19 = icmp_imm ne v18, 0
//     v20 = sshr v16, v15  ; v15 = 16
//     v21 = iconst.i64 1
//     v22 = iadd v20, v21  ; v21 = 1
//     v23 = select v19, v22, v20
//     v24 = ishl v23, v15  ; v15 = 16
//     v25 = ireduce.i32 v24
//     v26 = iconst.i64 16
//     v27 = sextend.i64 v13
//     v28 = iconst.i64 0xffff
//     v29 = band v27, v28  ; v28 = 0xffff
//     v30 = icmp_imm ne v29, 0
//     v31 = sshr v27, v26  ; v26 = 16
//     v32 = iconst.i64 1
//     v33 = iadd v31, v32  ; v32 = 1
//     v34 = select v30, v33, v31
//     v35 = ishl v34, v26  ; v26 = 16
//     v36 = ireduce.i32 v35
//     v37 = iconst.i64 16
//     v38 = sextend.i64 v14  ; v14 = 6554
//     v39 = iconst.i64 0xffff
//     v40 = band v38, v39  ; v39 = 0xffff
//     v41 = icmp_imm ne v40, 0
//     v42 = sshr v38, v37  ; v37 = 16
//     v43 = iconst.i64 1
//     v44 = iadd v42, v43  ; v43 = 1
//     v45 = select v41, v44, v42
//     v46 = ishl v45, v37  ; v37 = 16
//     v47 = ireduce.i32 v46
//     store notrap aligned v25, v0
//     store notrap aligned v36, v0+4
//     store notrap aligned v47, v0+8
//     return
//
// block1:
//     v48 = iconst.i32 0
//     store notrap aligned v48, v0  ; v48 = 0
//     v49 = iconst.i32 0
//     store notrap aligned v49, v0+4  ; v49 = 0
//     v50 = iconst.i32 0
//     store notrap aligned v50, v0+8  ; v50 = 0
//     return
// }
// run: ≈ vec3(4, -2, 1) (tolerance: 0.01)
