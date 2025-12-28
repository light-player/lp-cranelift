// test run
// target riscv32.fixed32

// Phase 8: Constant Expression Array Sizes - Support constant expressions for array sizes

int phase8() {
    // Constant variable
    const int n = 5;
    int arr1[n];
    arr1[0] = 10;
    arr1[4] = 50;
    
    // Constant expression
    int arr2[3 + 2]; // Size 5
    arr2[0] = 1;
    arr2[4] = 5;
    
    // Multiple constants
    const int a = 2;
    const int b = 3;
    int arr3[a * b]; // Size 6
    arr3[0] = 100;
    arr3[5] = 600;
    
    return arr1[0] + arr1[4] + arr2[0] + arr2[4] + arr3[0] + arr3[5];
    // 10 + 50 + 1 + 5 + 100 + 600 = 766
}

// run: phase8() == 766

