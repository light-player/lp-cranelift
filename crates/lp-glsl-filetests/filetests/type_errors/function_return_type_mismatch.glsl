// test error

int get_int() {
    return 3.14;  // ERROR: float→int not allowed
}

int main() {
    return get_int();
}

// EXPECT_ERROR: Compilation error: Compilation error: Verifier errors

