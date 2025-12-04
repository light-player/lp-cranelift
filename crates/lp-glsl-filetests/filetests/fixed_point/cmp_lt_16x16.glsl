// test compile
// test fixed16

int main() {
    float a = 2.5;
    float b = 3.5;
    if (a < b) {
        return 1;
    }
    return 0;
}

// CHECK: icmp slt
// CHECK-NOT: fcmp

