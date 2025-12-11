// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return tanh(0.0);
}

// function u0:0() -> i32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = %tanhf sig0
//
// block0:
//     v3 = iconst.i32 0
//     v4 = iconst.i32 0
//     v5 = iconst.i32 1
//     v6 = icmp slt v3, v4  ; v3 = 0, v4 = 0
//     v7 = iabs v3  ; v3 = 0
//     v8 = iadd v5, v7  ; v5 = 1
//     v9 = sextend.i64 v3  ; v3 = 0
//     v10 = iconst.i64 16
//     v11 = ishl v9, v10  ; v10 = 16
//     v12 = sextend.i64 v8
//     v13 = sdiv v11, v12
//     v14 = ireduce.i32 v13
//     v15 = iconst.i32 0x0002_0000
//     v16 = icmp sgt v7, v15  ; v15 = 0x0002_0000
//     v17 = ineg v5  ; v5 = 1
//     v18 = select v6, v17, v5  ; v5 = 1
//     v19 = select v16, v18, v14
//     return v19
//
// block1:
//     v20 = iconst.i32 0
//     return v20  ; v20 = 0
// }
// run: ~= 0.0 
