// test run
// target riscv32.fixed32

// Phase 5: Multi-dimensional Arrays - Nested arrays with multi-dimensional indexing

int phase5() {
    // 2D array
    int arr[3][2];
    
    // Initialize
    arr[0][0] = 1;
    arr[0][1] = 2;
    arr[1][0] = 3;
    arr[1][1] = 4;
    arr[2][0] = 5;
    arr[2][1] = 6;
    
    // Read from 2D array
    int x = arr[0][0]; // 1
    int y = arr[1][1]; // 4
    int z = arr[2][0]; // 5
    
    return x + y + z; // 1 + 4 + 5 = 10
}

// run: phase5() == 10

