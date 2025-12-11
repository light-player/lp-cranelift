// test compile
// test run

vec4 main() {
    vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
    color.rgb = vec3(1.0, 0.5, 0.25);
    return color;  // Should return vec4(1.0, 0.5, 0.25, 1.0)
}

// run: == vec4(1.0, 0.5, 0.25, 1.0)





