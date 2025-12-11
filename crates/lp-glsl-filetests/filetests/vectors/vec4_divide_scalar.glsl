// test compile

// target riscv32.fixed32
int main() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 halved = v / 2.0;  // (5.0, 10.0, 15.0, 20.0)
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x000a_0000
//     v1 = iconst.i32 0x0014_0000
//     v2 = iconst.i32 0x001e_0000
//     v3 = iconst.i32 0x0028_0000
//     v4 = iconst.i32 0x0002_0000
//     v5 = iconst.i32 0
//     v6 = icmp eq v4, v5  ; v4 = 0x0002_0000, v5 = 0
//     v7 = iconst.i32 0x7fff_0000
//     v8 = iconst.i32 -2147483648
//     v9 = icmp eq v0, v5  ; v0 = 0x000a_0000, v5 = 0
//     v10 = icmp slt v0, v5  ; v0 = 0x000a_0000, v5 = 0
//     v11 = select v10, v8, v7  ; v8 = -2147483648, v7 = 0x7fff_0000
//     v12 = select v9, v5, v11  ; v5 = 0
//     v13 = iconst.i32 1
//     v14 = select v6, v13, v4  ; v13 = 1, v4 = 0x0002_0000
//     v15 = sextend.i64 v0  ; v0 = 0x000a_0000
//     v16 = iconst.i64 16
//     v17 = ishl v15, v16  ; v16 = 16
//     v18 = sextend.i64 v14
//     v19 = sdiv v17, v18
//     v20 = ireduce.i32 v19
//     v21 = select v6, v12, v20
//     v22 = iconst.i32 0
//     v23 = icmp eq v4, v22  ; v4 = 0x0002_0000, v22 = 0
//     v24 = iconst.i32 0x7fff_0000
//     v25 = iconst.i32 -2147483648
//     v26 = icmp eq v1, v22  ; v1 = 0x0014_0000, v22 = 0
//     v27 = icmp slt v1, v22  ; v1 = 0x0014_0000, v22 = 0
//     v28 = select v27, v25, v24  ; v25 = -2147483648, v24 = 0x7fff_0000
//     v29 = select v26, v22, v28  ; v22 = 0
//     v30 = iconst.i32 1
//     v31 = select v23, v30, v4  ; v30 = 1, v4 = 0x0002_0000
//     v32 = sextend.i64 v1  ; v1 = 0x0014_0000
//     v33 = iconst.i64 16
//     v34 = ishl v32, v33  ; v33 = 16
//     v35 = sextend.i64 v31
//     v36 = sdiv v34, v35
//     v37 = ireduce.i32 v36
//     v38 = select v23, v29, v37
//     v39 = iconst.i32 0
//     v40 = icmp eq v4, v39  ; v4 = 0x0002_0000, v39 = 0
//     v41 = iconst.i32 0x7fff_0000
//     v42 = iconst.i32 -2147483648
//     v43 = icmp eq v2, v39  ; v2 = 0x001e_0000, v39 = 0
//     v44 = icmp slt v2, v39  ; v2 = 0x001e_0000, v39 = 0
//     v45 = select v44, v42, v41  ; v42 = -2147483648, v41 = 0x7fff_0000
//     v46 = select v43, v39, v45  ; v39 = 0
//     v47 = iconst.i32 1
//     v48 = select v40, v47, v4  ; v47 = 1, v4 = 0x0002_0000
//     v49 = sextend.i64 v2  ; v2 = 0x001e_0000
//     v50 = iconst.i64 16
//     v51 = ishl v49, v50  ; v50 = 16
//     v52 = sextend.i64 v48
//     v53 = sdiv v51, v52
//     v54 = ireduce.i32 v53
//     v55 = select v40, v46, v54
//     v56 = iconst.i32 0
//     v57 = icmp eq v4, v56  ; v4 = 0x0002_0000, v56 = 0
//     v58 = iconst.i32 0x7fff_0000
//     v59 = iconst.i32 -2147483648
//     v60 = icmp eq v3, v56  ; v3 = 0x0028_0000, v56 = 0
//     v61 = icmp slt v3, v56  ; v3 = 0x0028_0000, v56 = 0
//     v62 = select v61, v59, v58  ; v59 = -2147483648, v58 = 0x7fff_0000
//     v63 = select v60, v56, v62  ; v56 = 0
//     v64 = iconst.i32 1
//     v65 = select v57, v64, v4  ; v64 = 1, v4 = 0x0002_0000
//     v66 = sextend.i64 v3  ; v3 = 0x0028_0000
//     v67 = iconst.i64 16
//     v68 = ishl v66, v67  ; v67 = 16
//     v69 = sextend.i64 v65
//     v70 = sdiv v68, v69
//     v71 = ireduce.i32 v70
//     v72 = select v57, v63, v71
//     v73 = iconst.i32 1
//     return v73  ; v73 = 1
//
// block1:
//     v74 = iconst.i32 0
//     return v74  ; v74 = 0
// }


