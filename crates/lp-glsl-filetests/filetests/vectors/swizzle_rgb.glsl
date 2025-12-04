// test compile
// test run

vec3 main() {
    vec4 color = vec4(1.0, 0.5, 0.25, 1.0);
    return color.rgb;  // Should return vec3(1.0, 0.5, 0.25)
}

// run: == vec3(1.0, 0.5, 0.25)

