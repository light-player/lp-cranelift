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
//     v0 = iconst.i32 0x0005_0000
//     v1 = iconst.i32 0x0005_0000
//     v2 = iconst.i32 0x0005_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0005_0000
//     v5 = iconst.i32 0x0007_0000
//     v6 = iconst.i32 0
//     v7 = iconst.i32 0x0001_0000
//     v8 = icmp slt v3, v0  ; v3 = 0x0003_0000, v0 = 0x0005_0000
//     v9 = sextend.i32 v8
//     v10 = select v9, v6, v7  ; v6 = 0, v7 = 0x0001_0000
//     v11 = icmp slt v4, v1  ; v4 = 0x0005_0000, v1 = 0x0005_0000
//     v12 = sextend.i32 v11
//     v13 = select v12, v6, v7  ; v6 = 0, v7 = 0x0001_0000
//     v14 = icmp slt v5, v2  ; v5 = 0x0007_0000, v2 = 0x0005_0000
//     v15 = sextend.i32 v14
//     v16 = select v15, v6, v7  ; v6 = 0, v7 = 0x0001_0000
//     v17 = iadd v10, v13
//     v18 = iadd v17, v16
//     v19 = iconst.i32 0x0001_fd71
//     v20 = icmp sgt v18, v19  ; v19 = 0x0001_fd71
//     v21 = sextend.i32 v20
//     v22 = iconst.i8 1
//     v23 = iconst.i8 0
//     v24 = select v21, v22, v23  ; v22 = 1, v23 = 0
//     v25 = iconst.i32 0x0002_028f
//     v26 = icmp slt v18, v25  ; v25 = 0x0002_028f
//     v27 = sextend.i32 v26
//     v28 = iconst.i8 1
//     v29 = iconst.i8 0
//     v30 = select v27, v28, v29  ; v28 = 1, v29 = 0
//     v31 = iconst.i8 0
//     v32 = iconst.i8 1
//     v33 = icmp ne v24, v31  ; v31 = 0
//     v34 = icmp ne v30, v31  ; v31 = 0
//     v35 = select v34, v32, v31  ; v32 = 1, v31 = 0
//     v36 = select v33, v35, v31  ; v31 = 0
//     return v36
//
// block1:
//     v37 = iconst.i8 0
//     return v37  ; v37 = 0
// }
// run: == true
