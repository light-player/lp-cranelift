// test compile

int main() {
    ivec3 i = ivec3(1, 2, 3);
    vec3 v = vec3(i);  // Type conversion
    return 1;
}

// CHECK: fcvt_from_sint

