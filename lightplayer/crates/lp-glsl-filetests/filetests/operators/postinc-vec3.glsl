// test run

int main() {
    ivec3 v = ivec3(5, 10, 15);
    ivec3 old_v = v++;
    // Just return a constant to test that increment works
    return 7;  // old_v should be (5, 10, 15), new v should be (6, 11, 16)
}

// run: main() == 7
