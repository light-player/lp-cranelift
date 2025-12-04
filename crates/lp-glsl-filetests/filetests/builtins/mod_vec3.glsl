// test compile
// test run

bool main() {
    vec3 result = mod(vec3(7.0, 8.0, 9.0), vec3(3.0, 3.0, 4.0));  // (1.0, 2.0, 1.0)
    // Validate: sum = 1 + 2 + 1 = 4.0
    float sum = result.x + result.y + result.z;
    return sum > 3.99 && sum < 4.01;
}

// run: == true

// Verify component-wise mod
// CHECK-LABEL: function u0:0
// CHECK: fdiv   ; x[0]/y[0]
// CHECK: floor  ; floor(x[0]/y[0])
// CHECK: fmul   ; y[0]*floor(x[0]/y[0])
// CHECK: fsub   ; mod result[0]
// CHECK: fdiv   ; x[1]/y[1]
// CHECK: floor
// CHECK: fmul
// CHECK: fsub   ; mod result[1]

