// test run
// target riscv32.fixed32

// Test main() function with parameters (like fragment shader)
int main(int x, int y) {
    // Use pixel coordinates as seed for pattern generation
    // Scale coordinates to reasonable range for computation
    int seed_x = x * 10 + 100;
    int seed_y = y * 10 + 100;
    
    // Pattern generation through iterative computation
    int result = 0;
    int iterations = 50;
    
    // Iterative pattern calculation
    for (int i = 0; i < iterations; i = i + 1) {
        // Complex arithmetic operations
        int temp = seed_x * seed_x + seed_y * seed_y;
        result = result + (temp / 1000);
        
        // Update coordinates for next iteration
        int new_x = (seed_x * seed_x - seed_y * seed_y) / 100 + 200;
        int new_y = (2 * seed_x * seed_y) / 100 + 150;
        seed_x = new_x;
        seed_y = new_y;
        
        // Early exit if value gets too large
        if (result > 10000) {
            break;
        }
    }
    
    // Normalize result to a reasonable range (0-999)
    result = result % 1000;
    
    return result;
}

// run: main(0, 0) == <expected>

