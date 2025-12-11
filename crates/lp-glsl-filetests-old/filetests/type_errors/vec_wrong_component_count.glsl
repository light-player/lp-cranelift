// test error

int main() {
    vec3 v = vec3(1.0, 2.0);  // ERROR: need 3 components
    return 1;
}

// EXPECT_ERROR_CODE: E0115
// EXPECT_ERROR: `vec3` constructor has wrong number of components
// EXPECT_LOCATION: 3:14

