// test error

vec3 scale_vec(vec3 v, float s) {
    return v * s;
}

float main() {
    return scale_vec(5.0, 2.0);  // ERROR: first arg should be vec3
}

// EXPECT_ERROR: No matching function

