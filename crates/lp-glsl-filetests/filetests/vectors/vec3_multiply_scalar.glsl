// test compile

int main() {
    vec3 v = vec3(2.0, 3.0, 4.0);
    vec3 scaled = v * 2.0;  // (4.0, 6.0, 8.0)
    return 1;
}

// CHECK: fmul

