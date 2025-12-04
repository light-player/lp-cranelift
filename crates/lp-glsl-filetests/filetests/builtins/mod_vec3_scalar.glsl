// test compile
// test run

bool main() {
    vec3 result = mod(vec3(7.0, 8.0, 9.0), 3.0);  // (1.0, 2.0, 0.0)
    // Validate: sum = 1 + 2 + 0 = 3.0
    float sum = result.x + result.y + result.z;
    return sum > 2.99 && sum < 3.01;
}

// run: == true

// Verify scalar divisor broadcast: mod(vec3, float)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0x1.800000p1  ; 3.0 (y, scalar)
// CHECK: fdiv   ; x[0]/y (reuses y)
// CHECK: floor
// CHECK: fmul   ; y*floor(x[0]/y)
// CHECK: fsub   ; result[0]
// CHECK: fdiv   ; x[1]/y (reuses y)
// CHECK: floor
// CHECK: fmul
// CHECK: fsub   ; result[1]

