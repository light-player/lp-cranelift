// test compile
// test run

float main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    return v.xyz.y;  // vec3(1.0, 2.0, 3.0).y = 2.0
}

// run: == 2.0



