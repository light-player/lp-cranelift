// test compile
// test run

bool main() {
    float result = fract(3.75);  // 3.75 - floor(3.75) = 3.75 - 3.0 = 0.75
    return result > 0.74 && result < 0.76;
}

// run: == true

// Verify fract(x) = x - floor(x)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0x1.e00000p1  ; 3.75
// CHECK: v{{[0-9]+}} = floor
// CHECK: v{{[0-9]+}} = fsub  ; x - floor(x)

