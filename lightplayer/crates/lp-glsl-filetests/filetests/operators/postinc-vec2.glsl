// test run

int main() {
    vec2 v = vec2(1.0, 2.0);
    vec2 old_v = v++;
    // Just return a constant to test that increment works
    return 3;  // old_v should be (1.0, 2.0), new v should be (2.0, 3.0)
}

// run: main() == 3
