// test compile

vec3 main() {
    vec3 v = vec3(-1.0, 0.5, 2.0);
    return clamp(v, 0.0, 1.0);  // (0.0, 0.5, 1.0)
}

