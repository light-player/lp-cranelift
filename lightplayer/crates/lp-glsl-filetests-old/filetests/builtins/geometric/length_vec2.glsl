// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    vec2 v = vec2(3.0, 4.0);
    return length(v);  // sqrt(9 + 16) = 5.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v7 = iconst.i32 0x0003_0000
//     v8 = iconst.i32 0x0004_0000
//     v9 = sextend.i64 v7  ; v7 = 0x0003_0000
//     v10 = sextend.i64 v7  ; v7 = 0x0003_0000
//     v11 = imul v9, v10
//     v12 = iconst.i64 16
//     v13 = sshr v11, v12  ; v12 = 16
//     v14 = ireduce.i32 v13
//     v15 = sextend.i64 v8  ; v8 = 0x0004_0000
//     v16 = sextend.i64 v8  ; v8 = 0x0004_0000
//     v17 = imul v15, v16
//     v18 = iconst.i64 16
//     v19 = sshr v17, v18  ; v18 = 16
//     v20 = ireduce.i32 v19
//     v21 = iadd v14, v20
//     v22 = iconst.i32 0
//     v23 = icmp eq v21, v22  ; v22 = 0
//     v24 = iconst.i32 8
//     v25 = sshr v21, v24  ; v24 = 8
//     v26 = sextend.i64 v21
//     v27 = sextend.i64 v25
//     v28 = ishl v26, v24  ; v24 = 8
//     v29 = sdiv v28, v27
//     v30 = iadd v27, v29
//     v31 = iconst.i64 1
//     v32 = sshr v30, v31  ; v31 = 1
//     v33 = sdiv v28, v32
//     v34 = iadd v32, v33
//     v35 = iconst.i64 1
//     v36 = sshr v34, v35  ; v35 = 1
//     v37 = sdiv v28, v36
//     v38 = iadd v36, v37
//     v39 = iconst.i64 1
//     v40 = sshr v38, v39  ; v39 = 1
//     v41 = sdiv v28, v40
//     v42 = iadd v40, v41
//     v43 = iconst.i64 1
//     v44 = sshr v42, v43  ; v43 = 1
//     v45 = sdiv v28, v44
//     v46 = iadd v44, v45
//     v47 = iconst.i64 1
//     v48 = sshr v46, v47  ; v47 = 1
//     v49 = ireduce.i32 v48
//     v50 = select v23, v22, v49  ; v22 = 0
//     return v50
//
// block1:
//     v51 = iconst.i32 0
//     return v51  ; v51 = 0
// }
// run: ~= 5
