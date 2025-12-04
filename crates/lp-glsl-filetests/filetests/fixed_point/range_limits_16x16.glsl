// test compile  
// test fixed16

float main() {
    float a = 32767.0;
    return a - 32766.0;
}

// CHECK: iconst
// CHECK: isub

