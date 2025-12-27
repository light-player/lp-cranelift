// Phase 1: Foundation - Basic 1D scalar arrays with literal int sizes
// Stack allocation, pointer-based storage, basic read/write access

int main() {
    // Basic declaration
    int arr[5];
    
    // Basic write
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    arr[3] = 40;
    arr[4] = 50;
    
    // Basic read
    int x = arr[0];
    int y = arr[2];
    int z = arr[4];
    
    // Return sum to verify values
    return x + y + z; // Should be 10 + 30 + 50 = 90
}

