// test run
// target riscv32.fixed32

// Phase 3: Initialization - Array initializer lists, full/partial initialization, unsized arrays

int phase3() {
    // Full initialization
    int arr1[3] = {10, 20, 30};
    int x = arr1[0] + arr1[1] + arr1[2]; // 60
    
    // Partial initialization (remaining should be zero)
    int arr2[5] = {1, 2, 3};
    int y = arr2[0] + arr2[4]; // 1 + 0 = 1
    
    // Unsized array with inferred size from initializer
    int arr3[] = {100, 200, 300};
    int z = arr3[0] + arr3[2]; // 100 + 300 = 400
    
    return x + y + z; // 60 + 1 + 400 = 461
}

// run: phase3() == 461

