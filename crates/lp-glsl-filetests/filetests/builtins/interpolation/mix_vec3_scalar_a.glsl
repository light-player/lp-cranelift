// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, 0.25);  // 0*0.75 + vec3(10,20,30)*0.25 = (2.5, 5.0, 7.5)
    // Validate: sum = 2.5 + 5.0 + 7.5 = 15.0
    float sum = result.x + result.y + result.z;
    return sum > 14.99 && sum < 15.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0x000a_0000
//     v4 = iconst.i32 0x0014_0000
//     v5 = iconst.i32 0x001e_0000
//     v6 = iconst.i32 0x4000
//     v7 = iconst.i32 0x0001_0000
//     v8 = isub v7, v6  ; v7 = 0x0001_0000, v6 = 0x4000
//     v9 = sextend.i64 v0  ; v0 = 0
//     v10 = sextend.i64 v8
//     v11 = imul v9, v10
//     v12 = iconst.i64 16
//     v13 = sshr v11, v12  ; v12 = 16
//     v14 = ireduce.i32 v13
//     v15 = sextend.i64 v3  ; v3 = 0x000a_0000
//     v16 = sextend.i64 v6  ; v6 = 0x4000
//     v17 = imul v15, v16
//     v18 = iconst.i64 16
//     v19 = sshr v17, v18  ; v18 = 16
//     v20 = ireduce.i32 v19
//     v21 = iadd v14, v20
//     v22 = sextend.i64 v1  ; v1 = 0
//     v23 = sextend.i64 v8
//     v24 = imul v22, v23
//     v25 = iconst.i64 16
//     v26 = sshr v24, v25  ; v25 = 16
//     v27 = ireduce.i32 v26
//     v28 = sextend.i64 v4  ; v4 = 0x0014_0000
//     v29 = sextend.i64 v6  ; v6 = 0x4000
//     v30 = imul v28, v29
//     v31 = iconst.i64 16
//     v32 = sshr v30, v31  ; v31 = 16
//     v33 = ireduce.i32 v32
//     v34 = iadd v27, v33
//     v35 = sextend.i64 v2  ; v2 = 0
//     v36 = sextend.i64 v8
//     v37 = imul v35, v36
//     v38 = iconst.i64 16
//     v39 = sshr v37, v38  ; v38 = 16
//     v40 = ireduce.i32 v39
//     v41 = sextend.i64 v5  ; v5 = 0x001e_0000
//     v42 = sextend.i64 v6  ; v6 = 0x4000
//     v43 = imul v41, v42
//     v44 = iconst.i64 16
//     v45 = sshr v43, v44  ; v44 = 16
//     v46 = ireduce.i32 v45
//     v47 = iadd v40, v46
//     v48 = iadd v21, v34
//     v49 = iadd v48, v47
//     v50 = iconst.i32 0x000e_fd71
//     v51 = icmp sgt v49, v50  ; v50 = 0x000e_fd71
//     v52 = sextend.i32 v51
//     v53 = iconst.i8 1
//     v54 = iconst.i8 0
//     v55 = select v52, v53, v54  ; v53 = 1, v54 = 0
//     v56 = iconst.i32 0x000f_028f
//     v57 = icmp slt v49, v56  ; v56 = 0x000f_028f
//     v58 = sextend.i32 v57
//     v59 = iconst.i8 1
//     v60 = iconst.i8 0
//     v61 = select v58, v59, v60  ; v59 = 1, v60 = 0
//     v62 = iconst.i8 0
//     v63 = iconst.i8 1
//     v64 = icmp ne v55, v62  ; v62 = 0
//     v65 = icmp ne v61, v62  ; v62 = 0
//     v66 = select v65, v63, v62  ; v63 = 1, v62 = 0
//     v67 = select v64, v66, v62  ; v62 = 0
//     return v67
//
// block1:
//     v68 = iconst.i8 0
//     return v68  ; v68 = 0
// }
// run: == true
