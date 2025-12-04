// test compile

int main() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 halved = v / 2.0;  // (5.0, 10.0, 15.0, 20.0)
    return 1;
}

// CHECK: fdiv

