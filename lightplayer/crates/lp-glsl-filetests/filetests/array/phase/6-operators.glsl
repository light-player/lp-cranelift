// test run
// target riscv32.fixed32

// Phase 6: Verify All Operators - Increment/decrement, compound assignment, binary/unary ops

int phase6() {
    int arr[3] = {10, 20, 30};
    
    // Increment/decrement
    arr[0]++;        // arr[0] becomes 11
    ++arr[1];        // arr[1] becomes 21
    arr[2]--;        // arr[2] becomes 29
    --arr[0];        // arr[0] becomes 10
    
    // Compound assignment
    arr[0] += 5;     // arr[0] becomes 15
    arr[1] -= 3;     // arr[1] becomes 18
    arr[2] *= 2;     // arr[2] becomes 58
    
    // Binary operations
    int x = arr[0] + arr[1];  // 15 + 18 = 33
    int y = arr[2] - arr[0];  // 58 - 15 = 43
    int z = arr[0] * 2;       // 15 * 2 = 30
    
    // Unary operations
    int a = -arr[0];          // -15
    int b = +arr[1];          // 18
    
    return x + y + z + a + b; // 33 + 43 + 30 - 15 + 18 = 109
}

// run: phase6() == 109

