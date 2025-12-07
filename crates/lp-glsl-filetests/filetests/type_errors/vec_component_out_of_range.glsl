// test error

int main() {
    vec2 v = vec2(1.0, 2.0);
    float z = v.z;  // ERROR: vec2 has no z component
    return 1;
}

// EXPECT_ERROR_CODE: E0111
// EXPECT_ERROR: component 'z' not valid for vector with 2 components
// EXPECT_LOCATION: 4:16



