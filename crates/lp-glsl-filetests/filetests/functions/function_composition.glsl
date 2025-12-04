// test compile

float double_it(float x) {
    return x * 2.0;
}

float triple_it(float x) {
    return x * 3.0;
}

float main() {
    float a = double_it(5.0);   // 10.0
    float b = triple_it(a);     // 30.0
    return b;
}

