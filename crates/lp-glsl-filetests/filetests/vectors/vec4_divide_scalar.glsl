// test compile

// target riscv32.fixed32
int main() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 halved = v / 2.0;  // (5.0, 10.0, 15.0, 20.0)
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v11 = iconst.i32 0x000a_0000
//     v12 = iconst.i32 0x0014_0000
//     v13 = iconst.i32 0x001e_0000
//     v14 = iconst.i32 0x0028_0000
//     v15 = iconst.i32 0x0002_0000
//     v16 = sextend.i64 v11  ; v11 = 0x000a_0000
//     v17 = iconst.i64 16
//     v18 = ishl v16, v17  ; v17 = 16
//     v19 = sextend.i64 v15  ; v15 = 0x0002_0000
//     v20 = sdiv v18, v19
//     v21 = ireduce.i32 v20
//     v22 = sextend.i64 v12  ; v12 = 0x0014_0000
//     v23 = iconst.i64 16
//     v24 = ishl v22, v23  ; v23 = 16
//     v25 = sextend.i64 v15  ; v15 = 0x0002_0000
//     v26 = sdiv v24, v25
//     v27 = ireduce.i32 v26
//     v28 = sextend.i64 v13  ; v13 = 0x001e_0000
//     v29 = iconst.i64 16
//     v30 = ishl v28, v29  ; v29 = 16
//     v31 = sextend.i64 v15  ; v15 = 0x0002_0000
//     v32 = sdiv v30, v31
//     v33 = ireduce.i32 v32
//     v34 = sextend.i64 v14  ; v14 = 0x0028_0000
//     v35 = iconst.i64 16
//     v36 = ishl v34, v35  ; v35 = 16
//     v37 = sextend.i64 v15  ; v15 = 0x0002_0000
//     v38 = sdiv v36, v37
//     v39 = ireduce.i32 v38
//     v9 = iconst.i32 1
//     return v9  ; v9 = 1
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }

