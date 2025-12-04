// test compile
// test run

bool main() {
    vec3 edge = vec3(5.0, 5.0, 5.0);
    vec3 x = vec3(3.0, 5.0, 7.0);
    vec3 result = step(edge, x);  // (0.0, 1.0, 1.0) because 3<5, 5>=5, 7>5
    // Validate: sum = 0 + 1 + 1 = 2.0
    float sum = result.x + result.y + result.z;
    return sum > 1.99 && sum < 2.01;
}

// run: == true

// Verify component-wise step
// CHECK-LABEL: function u0:0
// CHECK: fcmp lt  ; x[0] < edge[0]
// CHECK: select   ; step[0]
// CHECK: fcmp lt  ; x[1] < edge[1]
// CHECK: select   ; step[1]
// CHECK: fcmp lt  ; x[2] < edge[2]
// CHECK: select   ; step[2]

