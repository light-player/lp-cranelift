// test compile

int main() {
    ivec2 a = ivec2(10, 20);
    ivec2 b = ivec2(3, 7);
    ivec2 c = a - b;  // (7, 13)
    return 1;
}

// CHECK: isub

