// test compile

float main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    v.y = 5.0;  // v = (1.0, 5.0, 3.0)
    return v.y;  // 5.0
}

