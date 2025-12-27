// Phase 9: Array Constructors - Array constructor syntax

int main() {
    // Explicit size constructor
    int arr1 = int[3](10, 20, 30);
    int x = arr1[0] + arr1[2]; // 10 + 30 = 40
    
    // Inferred size constructor
    int arr2 = int[](1, 2, 3, 4, 5);
    int y = arr2[0] + arr2[4]; // 1 + 5 = 6
    
    // Vector array constructor
    vec4 arr3 = vec4[2](vec4(1.0), vec4(2.0));
    float z = arr3[0].x + arr3[1].x; // 1.0 + 2.0 = 3.0
    
    return int(x + y + z); // 40 + 6 + 3 = 49
}

