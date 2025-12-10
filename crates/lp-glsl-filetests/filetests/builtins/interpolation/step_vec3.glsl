// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 edge = vec3(5.0, 5.0, 5.0);
    vec3 x = vec3(3.0, 5.0, 7.0);
    vec3 result = step(edge, x);  // (0.0, 1.0, 1.0) because 3<5, 5>=5, 7>5
    // Validate: sum = 0 + 1 + 1 = 2.0
    float sum = result.x + result.y + result.z;
    return sum > 1.99 && sum < 2.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v33 = iconst.i32 0x0005_0000
//     v34 = iconst.i32 0x0005_0000
//     v35 = iconst.i32 0x0005_0000
//     v36 = iconst.i32 0x0003_0000
//     v37 = iconst.i32 0x0005_0000
//     v38 = iconst.i32 0x0007_0000
//     v39 = iconst.i32 0
//     v40 = iconst.i32 0x0001_0000
//     v41 = icmp slt v36, v33  ; v36 = 0x0003_0000, v33 = 0x0005_0000
//     v42 = select v41, v39, v40  ; v39 = 0, v40 = 0x0001_0000
//     v43 = icmp slt v37, v34  ; v37 = 0x0005_0000, v34 = 0x0005_0000
//     v44 = select v43, v39, v40  ; v39 = 0, v40 = 0x0001_0000
//     v45 = icmp slt v38, v35  ; v38 = 0x0007_0000, v35 = 0x0005_0000
//     v46 = select v45, v39, v40  ; v39 = 0, v40 = 0x0001_0000
//     v47 = iadd v42, v44
//     v48 = iadd v47, v46
//     v49 = iconst.i32 0x0001_fd71
//     v50 = icmp sgt v48, v49  ; v49 = 0x0001_fd71
//     v18 = iconst.i8 1
//     v19 = iconst.i8 0
//     v20 = select v50, v18, v19  ; v18 = 1, v19 = 0
//     v51 = iconst.i32 0x0002_028f
//     v52 = icmp slt v48, v51  ; v51 = 0x0002_028f
//     v23 = iconst.i8 1
//     v24 = iconst.i8 0
//     v25 = select v52, v23, v24  ; v23 = 1, v24 = 0
//     v26 = iconst.i8 0
//     v27 = iconst.i8 1
//     v28 = icmp ne v20, v26  ; v26 = 0
//     v29 = icmp ne v25, v26  ; v26 = 0
//     v30 = select v29, v27, v26  ; v27 = 1, v26 = 0
//     v31 = select v28, v30, v26  ; v26 = 0
//     return v31
//
// block1:
//     v32 = iconst.i8 0
//     return v32  ; v32 = 0
// }
// run: == true
