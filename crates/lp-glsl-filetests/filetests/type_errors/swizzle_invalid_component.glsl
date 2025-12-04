// test compile

void main() {
    vec2 v = vec2(1.0, 2.0);
    float f = v.z;  // ERROR: vec2 only has x and y
}

// CHECK: error
// CHECK: not valid

