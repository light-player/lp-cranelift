// test compile

vec2 main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    return v.xg;  // ERROR: mixing xyzw with rgba
}

// CHECK: error
// CHECK: mix

