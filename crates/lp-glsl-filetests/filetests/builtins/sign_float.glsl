// test compile
// test run

bool main() {
    float a = sign(-5.0);  // -1.0
    float b = sign(0.0);   // 0.0
    float c = sign(5.0);   // 1.0
    // Validate sum: -1 + 0 + 1 = 0
    float sum = a + b + c;
    return sum > -0.01 && sum < 0.01;
}

// run: == true

// Verify sign(x): x>0 ? 1.0 : (x<0 ? -1.0 : 0.0)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const {{.*}}  ; -5.0
// CHECK: v{{[0-9]+}} = f32const 0.0
// CHECK: v{{[0-9]+}} = fcmp gt  ; x > 0
// CHECK: v{{[0-9]+}} = fcmp lt  ; x < 0
// CHECK: v{{[0-9]+}} = f32const 0x1.000000p0   ; 1.0
// CHECK: v{{[0-9]+}} = select   ; x>0 ? 1.0 : 0.0
// CHECK: v{{[0-9]+}} = f32const {{.*}}  ; -1.0
// CHECK: v{{[0-9]+}} = select   ; x<0 ? -1.0 : prev

