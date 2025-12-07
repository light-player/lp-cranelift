// test error

float main() {
    return atan(vec2(1.0), 2.0);  // ERROR: type mismatch
}
// EXPECT_ERROR: No matching overload for atan([Vec2, Float])

// EXPECT_ERROR_CODE: E0114
// EXPECT_LOCATION: 3:12
