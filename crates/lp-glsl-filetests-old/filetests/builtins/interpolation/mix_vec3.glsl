// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, vec3(0.5, 0.5, 0.5));  // (5.0, 10.0, 15.0)
    // Validate: x=5, y=10, z=15, sum=30
    float sum = result.x + result.y + result.z;
    return sum > 29.99 && sum < 30.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0x000a_0000
//     v4 = iconst.i32 0x0014_0000
//     v5 = iconst.i32 0x001e_0000
//     v6 = iconst.i32 0x8000
//     v7 = iconst.i32 0x8000
//     v8 = iconst.i32 0x8000
//     v9 = iconst.i32 0x0001_0000
//     v10 = isub v9, v6  ; v9 = 0x0001_0000, v6 = 0x8000
//     v11 = sextend.i64 v0  ; v0 = 0
//     v12 = sextend.i64 v10
//     v13 = imul v11, v12
//     v14 = iconst.i64 16
//     v15 = sshr v13, v14  ; v14 = 16
//     v16 = ireduce.i32 v15
//     v17 = sextend.i64 v3  ; v3 = 0x000a_0000
//     v18 = sextend.i64 v6  ; v6 = 0x8000
//     v19 = imul v17, v18
//     v20 = iconst.i64 16
//     v21 = sshr v19, v20  ; v20 = 16
//     v22 = ireduce.i32 v21
//     v23 = iadd v16, v22
//     v24 = iconst.i32 0x0001_0000
//     v25 = isub v24, v7  ; v24 = 0x0001_0000, v7 = 0x8000
//     v26 = sextend.i64 v1  ; v1 = 0
//     v27 = sextend.i64 v25
//     v28 = imul v26, v27
//     v29 = iconst.i64 16
//     v30 = sshr v28, v29  ; v29 = 16
//     v31 = ireduce.i32 v30
//     v32 = sextend.i64 v4  ; v4 = 0x0014_0000
//     v33 = sextend.i64 v7  ; v7 = 0x8000
//     v34 = imul v32, v33
//     v35 = iconst.i64 16
//     v36 = sshr v34, v35  ; v35 = 16
//     v37 = ireduce.i32 v36
//     v38 = iadd v31, v37
//     v39 = iconst.i32 0x0001_0000
//     v40 = isub v39, v8  ; v39 = 0x0001_0000, v8 = 0x8000
//     v41 = sextend.i64 v2  ; v2 = 0
//     v42 = sextend.i64 v40
//     v43 = imul v41, v42
//     v44 = iconst.i64 16
//     v45 = sshr v43, v44  ; v44 = 16
//     v46 = ireduce.i32 v45
//     v47 = sextend.i64 v5  ; v5 = 0x001e_0000
//     v48 = sextend.i64 v8  ; v8 = 0x8000
//     v49 = imul v47, v48
//     v50 = iconst.i64 16
//     v51 = sshr v49, v50  ; v50 = 16
//     v52 = ireduce.i32 v51
//     v53 = iadd v46, v52
//     v54 = iadd v23, v38
//     v55 = iadd v54, v53
//     v56 = iconst.i32 0x001d_fd71
//     v57 = icmp sgt v55, v56  ; v56 = 0x001d_fd71
//     v58 = sextend.i32 v57
//     v59 = iconst.i8 1
//     v60 = iconst.i8 0
//     v61 = select v58, v59, v60  ; v59 = 1, v60 = 0
//     v62 = iconst.i32 0x001e_028f
//     v63 = icmp slt v55, v62  ; v62 = 0x001e_028f
//     v64 = sextend.i32 v63
//     v65 = iconst.i8 1
//     v66 = iconst.i8 0
//     v67 = select v64, v65, v66  ; v65 = 1, v66 = 0
//     v68 = iconst.i8 0
//     v69 = iconst.i8 1
//     v70 = icmp ne v61, v68  ; v68 = 0
//     v71 = icmp ne v67, v68  ; v68 = 0
//     v72 = select v71, v69, v68  ; v69 = 1, v68 = 0
//     v73 = select v70, v72, v68  ; v68 = 0
//     return v73
//
// block1:
//     v74 = iconst.i8 0
//     return v74  ; v74 = 0
// }
// run: == true
