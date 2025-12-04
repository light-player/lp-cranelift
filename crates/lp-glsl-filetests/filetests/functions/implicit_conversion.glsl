// test compile

float add_floats(float a, float b) {
    return a + b;
}

float main() {
    int x = 5;
    return add_floats(x, 3.0);  // int→float conversion: 5.0 + 3.0 = 8.0
}

