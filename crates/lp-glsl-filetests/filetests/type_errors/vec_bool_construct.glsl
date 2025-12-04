// test error

int main() {
    vec3 v = vec3(true, false, true);  // ERROR: bool → float not allowed
    return 1;
}

// EXPECT_ERROR: Cannot use Bool in vec3 constructor

