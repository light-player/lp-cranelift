// test compile
// test run

vec2 main() {
    vec2 v = vec2(1.0, 2.0);
    v.yx = vec2(3.0, 4.0);  // Swaps: v.y = 3.0, v.x = 4.0
    return v;  // Should return vec2(4.0, 3.0)
}

// run: == vec2(4.0, 3.0)




