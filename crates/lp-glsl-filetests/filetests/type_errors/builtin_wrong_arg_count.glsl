// test error

float main() {
    return dot(vec3(1.0, 2.0, 3.0));  // ERROR: dot needs 2 args
}

// EXPECT_ERROR: No matching overload for dot

