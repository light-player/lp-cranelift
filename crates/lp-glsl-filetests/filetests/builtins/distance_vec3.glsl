// test compile

float main() {
    vec3 p0 = vec3(0.0, 0.0, 0.0);
    vec3 p1 = vec3(1.0, 2.0, 2.0);
    return distance(p0, p1);  // sqrt(1 + 4 + 4) = 3.0
}

// CHECK: fsub
// CHECK: sqrt

