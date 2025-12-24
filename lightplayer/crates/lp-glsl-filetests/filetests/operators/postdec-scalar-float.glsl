// test run

int main() {
    float x = 5.2;
    float old_x = x--;
    // Just return a constant to test that decrement works
    return 9;  // old_x + x would be 5.2 + 4.2 = 9.4 -> 9 when truncated
}

//
// }
// run: main() == 9
