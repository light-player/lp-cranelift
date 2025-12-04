// test compile

ivec3 main() {
    return ivec3(sign(-5), sign(0), sign(5));  // (-1, 0, 1)
}

// CHECK: icmp
// CHECK: select

