// test run
// target riscv32.fixed32

// Phase 2: Bounds Checking - Runtime bounds checking for array reads and writes

int phase2() {
    int arr[3];
    arr[0] = 1;
    arr[1] = 2;
    arr[2] = 3;
    
    // Valid access
    int x = arr[0];
    int y = arr[2];
    
    // These should trap at runtime (out of bounds):
    // int z = arr[3];  // Would trap
    // arr[-1] = 0;     // Would trap
    // arr[5] = 0;      // Would trap
    
    return x + y; // Should be 1 + 3 = 4
}

// run: phase2() == 4

