// test error

int main() {
    vec3 v = vec3(true, false, true);  // ERROR: cannot use bool in vec3 constructor
    return 0;
}

// EXPECT_ERROR_CODE: E0103
// EXPECT_ERROR: cannot use `Bool` in `vec3` constructor
// EXPECT_LOCATION: 3:14

