// test run

int main() {
    int x = 5;
    int old_x = x++;
    return old_x + x;  // Should return 5 + 6 = 11
}
// run: main() == 11
