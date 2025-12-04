// test error

int main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    v.x = vec2(4.0, 5.0);  // ERROR: assigning vector to scalar component
    return 1;
}

// EXPECT_ERROR: Component assignment requires scalar RHS

