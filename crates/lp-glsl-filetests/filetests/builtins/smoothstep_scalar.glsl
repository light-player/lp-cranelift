// test compile

float main() {
    // smoothstep at midpoint should be 0.5
    return smoothstep(0.0, 10.0, 5.0);
}

// CHECK: fmin
// CHECK: fmax

