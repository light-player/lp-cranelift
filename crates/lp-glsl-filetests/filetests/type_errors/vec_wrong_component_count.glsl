// test error

int main() {
    vec3 v = vec3(1.0, 2.0);  // ERROR: need 3 components
    return 1;
}

// EXPECT_ERROR: requires 3 components, got 2

