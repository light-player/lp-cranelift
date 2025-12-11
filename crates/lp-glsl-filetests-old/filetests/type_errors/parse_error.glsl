// test error

int main() {
    return 42
    // Missing semicolon - parse error
}

// EXPECT_ERROR_CODE: E0001
// EXPECT_ERROR: parse error
// EXPECT_LOCATION: 5:
// EXPECT_SPAN_TEXT: }

