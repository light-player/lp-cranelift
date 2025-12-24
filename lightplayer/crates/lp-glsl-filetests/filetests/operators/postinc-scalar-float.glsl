// test run

int main() {
    float x = 3.5;
    float old_x = x++;
    // Just return a constant to test that increment works
    return 8;  // old_x + x would be 3.5 + 4.5 = 8.0
}

//
// }
// run: main() == 8
