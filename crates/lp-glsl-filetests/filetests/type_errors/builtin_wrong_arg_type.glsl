// test error

int main() {
    float x = abs(true);  // ERROR: abs() requires numeric type, not bool
    return 1;
}

// EXPECT_ERROR_CODE: E0114
// EXPECT_ERROR: No matching overload for abs([Bool])
// EXPECT_LOCATION: 3:15
// EXPECT_SPAN_TEXT:     float x = abs(true);

