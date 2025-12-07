// test error

void main() {
    vec2 v = vec2(1.0, 2.0);
    float f = v.z;  // ERROR: vec2 only has x and y
}

// EXPECT_ERROR_CODE: E0111
// EXPECT_ERROR: component 'z' not valid for Vec2 (has only 2 components)
// EXPECT_LOCATION: 4:16



