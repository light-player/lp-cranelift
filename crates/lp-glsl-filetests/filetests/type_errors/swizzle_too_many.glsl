// test compile

void main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = v.xyzwx;  // ERROR: 5 components
}

// CHECK: error
// CHECK: most 4 components





