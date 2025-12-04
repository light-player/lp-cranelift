// test compile

vec3 main() {
    vec3 x = vec3(1.0, 0.0, 0.0);
    vec3 y = vec3(0.0, 1.0, 0.0);
    return cross(x, y);  // = (0.0, 0.0, 1.0)
}

// CHECK: fmul
// CHECK: fsub

