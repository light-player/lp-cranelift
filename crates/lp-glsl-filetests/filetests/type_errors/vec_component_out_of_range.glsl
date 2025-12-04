// test error

int main() {
    vec2 v = vec2(1.0, 2.0);
    float z = v.z;  // ERROR: vec2 has no z component
    return 1;
}

// EXPECT_ERROR: Component z not valid for Vec2

