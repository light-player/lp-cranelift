// test run
// target riscv32.fixed32

// A computationally expensive pattern generator shader
// Similar to a simplified fractal/mandelbrot - tests iterative computation
int pattern_generator() {
    // Simulate fragment shader coordinates (simplified)
    int x = 100;
    int y = 100;
    
    // Pattern generation through iterative computation
    // This is similar to what a real shader would do for effects like:
    // - Noise generation
    // - Pattern generation
    // - Simple raytracing
    int result = 0;
    int iterations = 50;  // Number of iterations for computation
    
    // Iterative pattern calculation (like a simplified mandelbrot/fractal)
    for (int i = 0; i < iterations; i = i + 1) {
        // Complex arithmetic operations
        int temp = x * x + y * y;
        result = result + (temp / 1000);
        
        // Update coordinates for next iteration
        int new_x = (x * x - y * y) / 100 + 200;
        int new_y = (2 * x * y) / 100 + 150;
        x = new_x;
        y = new_y;
        
        // Early exit if value gets too large (like escape condition)
        if (result > 10000) {
            break;
        }
    }
    
    // Normalize result to a reasonable range
    result = result % 1000;
    
    return result;
}

// run: pattern_generator() == 302

