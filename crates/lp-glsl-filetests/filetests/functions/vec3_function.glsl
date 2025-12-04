// test compile

vec3 scale(vec3 v, float s) {
    return v * s;
}

vec3 main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return scale(v, 2.0);  // (2.0, 4.0, 6.0)
}

