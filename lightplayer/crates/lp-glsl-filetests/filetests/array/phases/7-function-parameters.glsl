// Phase 7: Function Parameters - Arrays as function parameters and return values

int sum_array(int arr[5]) {
    return arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
}

int main() {
    int arr[5] = {1, 2, 3, 4, 5};
    
    // Pass array to function
    int result = sum_array(arr);
    
    return result; // Should be 15
}

