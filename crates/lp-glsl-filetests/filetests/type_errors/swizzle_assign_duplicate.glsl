// test compile

void main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xx = vec2(5.0, 6.0);  // ERROR: 'x' used twice
}

// CHECK: error
// CHECK: duplicate

