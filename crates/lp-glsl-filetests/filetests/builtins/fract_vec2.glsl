// test compile

vec2 main() {
    return fract(vec2(3.75, 5.25));  // (0.75, 0.25)
}

// CHECK: floor
// CHECK: fsub

