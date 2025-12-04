// test compile

void main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xy = vec3(5.0, 6.0, 7.0);  // ERROR: vec3 cannot assign to 2 components
}

// CHECK: error
// CHECK: mismatch

