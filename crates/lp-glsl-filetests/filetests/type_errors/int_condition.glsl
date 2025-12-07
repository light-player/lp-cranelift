// test error

int main() {
    int x = 5;
    if (x) {  // ERROR: condition must be bool
        return 1;
    }
    return 0;
}

// EXPECT_ERROR_CODE: E0107
// EXPECT_ERROR: condition must be bool type
// EXPECT_LOCATION: 4:9
// EXPECT_SPAN_TEXT:     if (x) {  // ERROR: condition must be bool

