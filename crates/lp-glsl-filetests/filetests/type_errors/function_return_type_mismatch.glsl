// test error

int get_int() {
    return 3.14;  // ERROR: float→int not allowed
}

int main() {
    return get_int();
}

// Note: This may produce a verification error rather than a semantic error
// Verifier errors don't preserve location information
// EXPECT_ERROR_CODE: E0116
// EXPECT_ERROR: return type mismatch: expected `Int`, found `Float`

// EXPECT_LOCATION: 3:12
