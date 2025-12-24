// test run

int main() {
    int x = 8;
    int old_x = x--;
    return old_x + x;  // Should return 8 + 7 = 15
}

// run: main() == 15
