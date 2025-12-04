// test compile
// test run

bool main() {
    vec2 result = fract(vec2(3.75, 5.25));  // (0.75, 0.25)
    // Validate: sum = 0.75 + 0.25 = 1.0
    float sum = result.x + result.y;
    return sum > 0.99 && sum < 1.01;
}

// run: == true

// Verify component-wise fract
// CHECK-LABEL: function u0:0
// CHECK: floor  ; floor(x[0])
// CHECK: fsub   ; x[0] - floor(x[0])
// CHECK: floor  ; floor(x[1])
// CHECK: fsub   ; x[1] - floor(x[1])

