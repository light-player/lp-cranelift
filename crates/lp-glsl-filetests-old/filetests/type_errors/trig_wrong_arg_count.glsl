// test error

float main() {
    return sin(1.0, 2.0);  // ERROR: too many args
}
// EXPECT_ERROR: No matching overload for sin

