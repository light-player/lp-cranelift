// test error

float main() {
    return sin(true);  // ERROR: bool not allowed
}
// EXPECT_ERROR: No matching overload for sin

