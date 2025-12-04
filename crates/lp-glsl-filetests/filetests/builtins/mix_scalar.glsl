// test compile
// test run

bool main() {
    float result = mix(0.0, 10.0, 0.5);  // Should return 5.0: 0*(1-0.5) + 10*0.5
    // Validate result is approximately 5.0
    return result > 4.99 && result < 5.01;
}

// run: == true

// Verify IR for mix(x, y, a) = x * (1-a) + y * a
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0.0
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p3  ; 10.0
// CHECK: v{{[0-9]+}} = f32const 0x1.000000p-1 ; 0.5
// CHECK: v{{[0-9]+}} = f32const 0x1.000000p0  ; 1.0
// CHECK: v{{[0-9]+}} = fsub
// CHECK: v{{[0-9]+}} = fmul  ; x * (1-a)
// CHECK: v{{[0-9]+}} = fmul  ; y * a
// CHECK: v{{[0-9]+}} = fadd  ; x*(1-a) + y*a

