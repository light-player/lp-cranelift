// Phase 4: Vector/Matrix Element Arrays - Arrays of vectors and matrices with component access

int main() {
    // Array of vectors
    vec4 arr[3];
    arr[0] = vec4(1.0, 2.0, 3.0, 4.0);
    arr[1] = vec4(5.0, 6.0, 7.0, 8.0);
    arr[2] = vec4(9.0, 10.0, 11.0, 12.0);
    
    // Component access
    float x = arr[0].x; // 1.0
    float y = arr[1].y; // 6.0
    float z = arr[2].z; // 11.0
    
    // Array of matrices
    mat3 mats[2];
    mats[0] = mat3(1.0);
    mats[1] = mat3(2.0);
    
    // Matrix element access
    float m = mats[0][0][0]; // 1.0
    float n = mats[1][1][1]; // 2.0
    
    return int(x + y + z + m + n); // 1 + 6 + 11 + 1 + 2 = 21
}

