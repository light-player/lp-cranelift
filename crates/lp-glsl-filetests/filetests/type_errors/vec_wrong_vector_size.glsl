// test error

int main() {
    vec3 v = vec3(vec2(1.0, 2.0));  // ERROR: vec2 has wrong size
    return 1;
}

// EXPECT_ERROR_CODE: E0115
// EXPECT_ERROR: cannot construct `vec3` from `Vec2`
// EXPECT_LOCATION: 3:14

