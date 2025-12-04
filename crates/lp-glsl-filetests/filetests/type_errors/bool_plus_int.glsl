// test error

int main() {
    int x = 5;
    bool y = true;
    return x + y;  // ERROR: cannot add int and bool
}

// EXPECT_ERROR: Arithmetic operator .* requires numeric operands

