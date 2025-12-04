// test compile
// test run

bool main() {
    float result = mod(7.0, 3.0);  // 7 - 3*floor(7/3) = 7 - 3*2 = 1.0
    return result > 0.99 && result < 1.01;
}

// run: == true

// Verify mod(x, y) = x - y * floor(x/y)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0x1.c00000p2  ; 7.0
// CHECK: v{{[0-9]+}} = f32const 0x1.800000p1  ; 3.0
// CHECK: v{{[0-9]+}} = fdiv   ; x / y
// CHECK: v{{[0-9]+}} = floor  ; floor(x/y)
// CHECK: v{{[0-9]+}} = fmul   ; y * floor(x/y)
// CHECK: v{{[0-9]+}} = fsub   ; x - y*floor(x/y)

